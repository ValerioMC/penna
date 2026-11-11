use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, WebviewUrl, WebviewWindowBuilder};

use super::placement::{place_near_anchor, ScreenArea, WindowSize};
use crate::model::error::PennaResult;
use crate::model::selection::ScreenPoint;

pub const POPUP_LABEL: &str = "popup";
const ICON_LABEL: &str = "icon";
const ICON_SIZE: WindowSize = WindowSize {
    width: 36.0,
    height: 36.0,
};
const POPUP_SIZE: WindowSize = WindowSize {
    width: 380.0,
    height: 260.0,
};

pub fn create_windows(app: &AppHandle) -> PennaResult<()> {
    create_overlay_window(app, ICON_LABEL, "icon.html", ICON_SIZE)?;
    create_overlay_window(app, POPUP_LABEL, "popup.html", POPUP_SIZE)?;
    Ok(())
}

pub fn show_icon(app: &AppHandle, anchor: ScreenPoint) -> PennaResult<()> {
    show_near(app, ICON_LABEL, ICON_SIZE, anchor)
}

pub fn hide_icon(app: &AppHandle) -> PennaResult<()> {
    hide(app, ICON_LABEL)
}

pub fn show_popup(app: &AppHandle, anchor: ScreenPoint) -> PennaResult<()> {
    show_near(app, POPUP_LABEL, POPUP_SIZE, anchor)
}

pub fn hide_popup(app: &AppHandle) -> PennaResult<()> {
    hide(app, POPUP_LABEL)
}

fn create_overlay_window(
    app: &AppHandle,
    label: &str,
    page: &str,
    size: WindowSize,
) -> PennaResult<()> {
    WebviewWindowBuilder::new(app, label, WebviewUrl::App(page.into()))
        .inner_size(size.width, size.height)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible_on_all_workspaces(true)
        .visible(false)
        .focused(false)
        .focusable(false)
        .accept_first_mouse(true)
        .build()?;
    Ok(())
}

fn show_near(
    app: &AppHandle,
    label: &str,
    size: WindowSize,
    anchor: ScreenPoint,
) -> PennaResult<()> {
    let Some(window) = app.get_webview_window(label) else {
        tracing::error!(label, "overlay window is missing");
        return Ok(());
    };
    let position = place_near_anchor(anchor, size, screen_containing(app, anchor)?);
    window.set_position(LogicalPosition::new(position.x, position.y))?;
    window.show()?;
    Ok(())
}

fn hide(app: &AppHandle, label: &str) -> PennaResult<()> {
    if let Some(window) = app.get_webview_window(label) {
        window.hide()?;
    }
    Ok(())
}

fn screen_containing(app: &AppHandle, point: ScreenPoint) -> PennaResult<Option<ScreenArea>> {
    for monitor in app.available_monitors()? {
        let scale = monitor.scale_factor();
        let position: LogicalPosition<f64> = monitor.position().to_logical(scale);
        let size: LogicalSize<f64> = monitor.size().to_logical(scale);
        let area = ScreenArea {
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
        };
        if area.contains(point) {
            return Ok(Some(area));
        }
    }
    Ok(None)
}
