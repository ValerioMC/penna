use crate::model::error::PennaResult;

pub const LATEST_DOWNLOAD_URL: &str =
    "https://github.com/ValerioMC/penna/releases/latest/download/Penna-macos-arm64.dmg";

pub fn open_latest_download() -> PennaResult<()> {
    open::that(LATEST_DOWNLOAD_URL)?;
    Ok(())
}
