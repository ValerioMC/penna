use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::AppHandle;

const QUIT_MENU_ID: &str = "quit";

pub fn build(app: &AppHandle) -> tauri::Result<TrayIcon> {
    let quit_item = MenuItem::with_id(app, QUIT_MENU_ID, "Quit Penna", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit_item])?;
    TrayIconBuilder::new()
        .icon(tray_icon())
        .tooltip("Penna")
        .menu(&menu)
        .on_menu_event(|app, event| {
            if event.id().as_ref() == QUIT_MENU_ID {
                app.exit(0);
            }
        })
        .build(app)
}

fn tray_icon() -> Image<'static> {
    tauri::include_image!("./tray.png")
}
