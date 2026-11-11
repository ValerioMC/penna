#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> Result<(), penna_lib::PennaError> {
    penna_lib::run()
}
