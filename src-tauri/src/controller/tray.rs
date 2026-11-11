use std::sync::Arc;

use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Manager};

use super::rewrite_selection_in_background;
use crate::service::rewrite_service::RewriteService;

const REWRITE_MENU_ID: &str = "rewrite-selection";
const QUIT_MENU_ID: &str = "quit";

pub fn build(app: &AppHandle) -> tauri::Result<TrayIcon> {
    let rewrite_item = MenuItem::with_id(
        app,
        REWRITE_MENU_ID,
        "Rewrite Selection",
        true,
        None::<&str>,
    )?;
    let quit_item = MenuItem::with_id(app, QUIT_MENU_ID, "Quit Penna", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&rewrite_item, &quit_item])?;
    TrayIconBuilder::new()
        .icon(tray_icon())
        .tooltip("Penna")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            REWRITE_MENU_ID => {
                let service = app.state::<Arc<RewriteService>>().inner().clone();
                rewrite_selection_in_background(service);
            }
            QUIT_MENU_ID => app.exit(0),
            _ => {}
        })
        .build(app)
}

fn tray_icon() -> Image<'static> {
    tauri::include_image!("./tray.png")
}
