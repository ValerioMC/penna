use std::collections::HashSet;
use std::ffi::c_void;
use std::ptr;

use accessibility_sys::{
    kAXBoundsForRangeParameterizedAttribute, kAXErrorSuccess, kAXFocusedApplicationAttribute,
    kAXFocusedUIElementAttribute, kAXFrontmostAttribute, kAXParentAttribute, kAXRoleAttribute,
    kAXSelectedTextAttribute, kAXSelectedTextRangeAttribute, kAXTrustedCheckOptionPrompt,
    kAXValueTypeCGRect, AXIsProcessTrusted, AXIsProcessTrustedWithOptions,
    AXUIElementCopyAttributeValue, AXUIElementCopyParameterizedAttributeValue,
    AXUIElementCreateApplication, AXUIElementCreateSystemWide, AXUIElementGetPid, AXUIElementRef,
    AXUIElementSetAttributeValue, AXValueGetValue, AXValueRef,
};
use core_foundation::base::{CFType, CFTypeRef, FromVoid, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use core_graphics::event::CGEvent;
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::window::{
    copy_window_info, kCGNullWindowID, kCGWindowLayer, kCGWindowListExcludeDesktopElements,
    kCGWindowListOptionOnScreenOnly, kCGWindowOwnerPID,
};

use super::selection_reader::SelectionReader;
use crate::model::error::{PennaError, PennaResult};
use crate::model::selection::{ScreenPoint, SelectionReading, SelectionSnapshot};

const BROWSER_ACCESSIBILITY_ATTRIBUTES: [&str; 2] =
    ["AXManualAccessibility", "AXEnhancedUserInterface"];
const SELECTED_TEXT_MARKER_RANGE_ATTRIBUTE: &str = "AXSelectedTextMarkerRange";
const STRING_FOR_TEXT_MARKER_RANGE_ATTRIBUTE: &str = "AXStringForTextMarkerRange";
const BOUNDS_FOR_TEXT_MARKER_RANGE_ATTRIBUTE: &str = "AXBoundsForTextMarkerRange";
const MAX_ANCESTORS_SEARCHED_FOR_WEB_SELECTION: usize = 8;

pub fn request_permission_prompt() -> bool {
    let prompt_key = unsafe { CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt) };
    let options = CFDictionary::from_CFType_pairs(&[(
        prompt_key.as_CFType(),
        CFBoolean::true_value().as_CFType(),
    )]);
    unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) }
}

pub fn bring_application_to_front(process_id: i32) -> PennaResult<()> {
    let application = AxElement::application(process_id);
    let status = application.set_boolean(kAXFrontmostAttribute, true);
    if status != kAXErrorSuccess {
        return Err(PennaError::ApplicationActivation { process_id, status });
    }
    Ok(())
}

/// Reads the text selected in the focused app through the macOS Accessibility API,
/// without touching the clipboard.
#[derive(Default)]
pub struct AccessibilitySelectionReader {
    prepared_processes: HashSet<i32>,
    last_diagnostic: Option<String>,
}

impl AccessibilitySelectionReader {
    fn prepare_application_once(&mut self, process_id: i32, application: &AxElement) {
        if self.prepared_processes.insert(process_id) {
            application.enable_browser_accessibility();
        }
    }

    fn log_diagnostic_once(&mut self, diagnostic: String) {
        if self.last_diagnostic.as_deref() != Some(diagnostic.as_str()) {
            tracing::info!(%diagnostic, "no selection could be read");
            self.last_diagnostic = Some(diagnostic);
        }
    }

    fn indeterminate(&mut self, diagnostic: String) -> PennaResult<SelectionReading> {
        self.log_diagnostic_once(diagnostic);
        Ok(SelectionReading::Indeterminate)
    }

    fn unreadable(&mut self, diagnostic: String) -> PennaResult<SelectionReading> {
        self.log_diagnostic_once(diagnostic);
        Ok(SelectionReading::Unreadable)
    }
}

impl SelectionReader for AccessibilitySelectionReader {
    fn read_selection(&mut self) -> PennaResult<SelectionReading> {
        if !unsafe { AXIsProcessTrusted() } {
            return Err(PennaError::AccessibilityNotGranted);
        }
        let system = AxElement::system_wide();
        let Some(application) = system
            .element(kAXFocusedApplicationAttribute)
            .or_else(frontmost_application)
        else {
            return self.indeterminate("no focused application".to_string());
        };
        let Some(process_id) = application.process_id() else {
            return self.indeterminate("the focused application has no process id".to_string());
        };
        if process_id == std::process::id() as i32 {
            return Ok(SelectionReading::Indeterminate);
        }
        self.prepare_application_once(process_id, &application);
        let Some(focused) = application.element(kAXFocusedUIElementAttribute) else {
            return self.unreadable(format!("process {process_id} exposes no focused element"));
        };
        let Some(selection) = focused.selection() else {
            let role = focused.text(kAXRoleAttribute).unwrap_or_default();
            return self.unreadable(format!(
                "process {process_id} focused element ({role}) exposes no selected text"
            ));
        };
        if !selection.has_text() {
            self.last_diagnostic = None;
            return Ok(SelectionReading::NothingSelected);
        }
        let anchor = match selection.bounds {
            Some(bounds) if bounds.is_usable() => bounds.bottom_right(),
            _ => mouse_location()?,
        };
        Ok(SelectionReading::Selected(SelectionSnapshot {
            process_id,
            text: selection.text,
            anchor,
        }))
    }
}

#[repr(C)]
#[derive(Default)]
struct Bounds {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl Bounds {
    fn is_usable(&self) -> bool {
        [self.x, self.y, self.width, self.height]
            .iter()
            .all(|value| value.is_finite())
            && (self.width > 0.0 || self.height > 0.0)
    }

    fn bottom_right(&self) -> ScreenPoint {
        ScreenPoint {
            x: self.x + self.width,
            y: self.y + self.height,
        }
    }
}

struct ElementSelection {
    text: String,
    bounds: Option<Bounds>,
}

impl ElementSelection {
    fn read(text: String, read_bounds: impl FnOnce() -> Option<Bounds>) -> Self {
        let bounds = if text.trim().is_empty() {
            None
        } else {
            read_bounds()
        };
        Self { text, bounds }
    }

    fn has_text(&self) -> bool {
        !self.text.trim().is_empty()
    }
}

struct AxElement(CFType);

impl AxElement {
    fn application(process_id: i32) -> Self {
        Self(unsafe {
            CFType::wrap_under_create_rule(AXUIElementCreateApplication(process_id) as CFTypeRef)
        })
    }

    fn system_wide() -> Self {
        Self(unsafe { CFType::wrap_under_create_rule(AXUIElementCreateSystemWide() as CFTypeRef) })
    }

    fn raw(&self) -> AXUIElementRef {
        self.0.as_CFTypeRef() as AXUIElementRef
    }

    fn attribute(&self, name: &str) -> Option<CFType> {
        let mut value: CFTypeRef = ptr::null();
        let status = unsafe {
            AXUIElementCopyAttributeValue(
                self.raw(),
                CFString::new(name).as_concrete_TypeRef(),
                &mut value,
            )
        };
        if status != kAXErrorSuccess || value.is_null() {
            return None;
        }
        Some(unsafe { CFType::wrap_under_create_rule(value) })
    }

    fn element(&self, name: &str) -> Option<AxElement> {
        self.attribute(name).map(Self)
    }

    fn text(&self, name: &str) -> Option<String> {
        self.attribute(name)?
            .downcast::<CFString>()
            .map(|text| text.to_string())
    }

    fn process_id(&self) -> Option<i32> {
        let mut process_id = 0;
        let status = unsafe { AXUIElementGetPid(self.raw(), &mut process_id) };
        (status == kAXErrorSuccess).then_some(process_id)
    }

    fn parameterized_attribute(&self, name: &str, parameter: &CFType) -> Option<CFType> {
        let mut value: CFTypeRef = ptr::null();
        let status = unsafe {
            AXUIElementCopyParameterizedAttributeValue(
                self.raw(),
                CFString::new(name).as_concrete_TypeRef(),
                parameter.as_CFTypeRef(),
                &mut value,
            )
        };
        if status != kAXErrorSuccess || value.is_null() {
            return None;
        }
        Some(unsafe { CFType::wrap_under_create_rule(value) })
    }

    fn bounds_for(&self, parameterized_attribute: &str, parameter: &CFType) -> Option<Bounds> {
        let value = self.parameterized_attribute(parameterized_attribute, parameter)?;
        let mut bounds = Bounds::default();
        let read = unsafe {
            AXValueGetValue(
                value.as_CFTypeRef() as AXValueRef,
                kAXValueTypeCGRect,
                &mut bounds as *mut Bounds as *mut c_void,
            )
        };
        read.then_some(bounds)
    }

    fn selection(&self) -> Option<ElementSelection> {
        let direct = self.direct_selection();
        if direct.as_ref().is_some_and(ElementSelection::has_text) {
            return direct;
        }
        self.web_selection().or(direct)
    }

    fn direct_selection(&self) -> Option<ElementSelection> {
        let text = self.text(kAXSelectedTextAttribute)?;
        Some(ElementSelection::read(text, || {
            let range = self.attribute(kAXSelectedTextRangeAttribute)?;
            self.bounds_for(kAXBoundsForRangeParameterizedAttribute, &range)
        }))
    }

    fn web_selection(&self) -> Option<ElementSelection> {
        let mut candidate = AxElement(self.0.clone());
        for _ in 0..=MAX_ANCESTORS_SEARCHED_FOR_WEB_SELECTION {
            if let Some(selection) = candidate.marker_selection() {
                return Some(selection);
            }
            candidate = candidate.element(kAXParentAttribute)?;
        }
        None
    }

    fn marker_selection(&self) -> Option<ElementSelection> {
        let range = self.attribute(SELECTED_TEXT_MARKER_RANGE_ATTRIBUTE)?;
        let text = self
            .parameterized_attribute(STRING_FOR_TEXT_MARKER_RANGE_ATTRIBUTE, &range)?
            .downcast::<CFString>()?
            .to_string();
        Some(ElementSelection::read(text, || {
            self.bounds_for(BOUNDS_FOR_TEXT_MARKER_RANGE_ATTRIBUTE, &range)
        }))
    }

    fn set_boolean(&self, name: &str, value: bool) -> i32 {
        let value = if value {
            CFBoolean::true_value()
        } else {
            CFBoolean::false_value()
        };
        unsafe {
            AXUIElementSetAttributeValue(
                self.raw(),
                CFString::new(name).as_concrete_TypeRef(),
                value.as_CFTypeRef(),
            )
        }
    }

    fn enable_browser_accessibility(&self) {
        for name in BROWSER_ACCESSIBILITY_ATTRIBUTES {
            let status = self.set_boolean(name, true);
            if status != kAXErrorSuccess {
                tracing::debug!(
                    attribute = name,
                    status,
                    "application ignored accessibility hint"
                );
            }
        }
    }
}

fn frontmost_application() -> Option<AxElement> {
    let windows = copy_window_info(
        kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
        kCGNullWindowID,
    )?;
    let own_process_id = std::process::id() as i32;
    let layer_key = unsafe { CFString::wrap_under_get_rule(kCGWindowLayer) };
    let process_id_key = unsafe { CFString::wrap_under_get_rule(kCGWindowOwnerPID) };
    windows
        .iter()
        .filter_map(|entry| {
            let window = unsafe { CFDictionary::<CFString, CFType>::from_void(*entry) };
            let layer = number_of(&window, &layer_key)?;
            let process_id = number_of(&window, &process_id_key)?;
            (layer == 0 && process_id != own_process_id).then_some(process_id)
        })
        .next()
        .map(AxElement::application)
}

fn number_of(window: &CFDictionary<CFString, CFType>, key: &CFString) -> Option<i32> {
    window.find(key)?.downcast::<CFNumber>()?.to_i32()
}

fn mouse_location() -> PennaResult<ScreenPoint> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| PennaError::MouseLocationUnavailable)?;
    let event = CGEvent::new(source).map_err(|_| PennaError::MouseLocationUnavailable)?;
    let location = event.location();
    Ok(ScreenPoint {
        x: location.x,
        y: location.y,
    })
}
