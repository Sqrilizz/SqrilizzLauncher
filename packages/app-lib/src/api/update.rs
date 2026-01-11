use reqwest;
use std::path::PathBuf;
use tokio::fs::File as AsyncFile;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub(crate) async fn get_resource(
    download_url: &str,
    local_filename: &str,
    os_type: &str,
    auto_update_supported: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let download_dir = dirs::download_dir()
        .ok_or("[AR] • Failed to determine download directory")?;
    let full_path = download_dir.join(local_filename);

    let response = reqwest::get(download_url).await?;
    let bytes = response.bytes().await?;
    let mut dest_file = AsyncFile::create(&full_path).await?;
    dest_file.write_all(&bytes).await?;
    tracing::info!("[AR] • File downloaded to: {:?}", full_path);

    if auto_update_supported {
        let result = match os_type.to_lowercase().as_str() {
            "windows" => handle_windows_file(&full_path).await,
            "macos" => open_macos_file(&full_path).await,
            _ => open_default(&full_path).await,
        };

        match result {
            Ok(_) => tracing::info!("[AR] • File opened successfully!"),
            Err(e) => tracing::info!("[AR] • Failed to open file: {e}"),
        }
    }

    Ok(())
}

async fn handle_windows_file(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or_default()
        .to_lowercase();

    if filename.ends_with(".exe") || filename.ends_with(".msi") {
        tracing::info!("[AR] • Detected installer: {}", filename);
        run_windows_installer(path).await
    } else {
        open_windows_folder(path).await
    }
}

async fn run_windows_installer(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let installer_path = path.to_str().unwrap_or_default();

    let status = if installer_path.ends_with(".msi") {
        Command::new("msiexec")
            .args(&["/i", installer_path, "/quiet"])
            .status()
            .await?
    } else {
        Command::new("cmd")
            .args(&["/C", installer_path])
            .status()
            .await?
    };

    if status.success() {
        tracing::info!("[AR] • Installer started successfully.");
        Ok(())
    } else {
        tracing::error!("Installer failed. Exit code: {:?}", status.code());
        tracing::info!("[AR] • Trying to open folder...");
        open_windows_folder(path).await
    }
}

async fn open_windows_folder(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let folder = path.parent().unwrap_or(path);
    let status = Command::new("explorer")
        .arg(folder.display().to_string())
        .status()
        .await?;

    if !status.success() {
        Err(format!("Exit code: {:?}", status.code()).into())
    } else {
        Ok(())
    }
}

async fn open_macos_file(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("open")
        .arg(path.to_str().unwrap_or_default())
        .status()
        .await?;

    if !status.success() {
        Err(format!("Exit code: {:?}", status.code()).into())
    } else {
        Ok(())
    }
}

async fn open_default(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new(".")
        .arg(path.to_str().unwrap_or_default())
        .status()
        .await?;

    if !status.success() {
        Err(format!("Exit code: {:?}", status.code()).into())
    } else {
        Ok(())
    }
}
