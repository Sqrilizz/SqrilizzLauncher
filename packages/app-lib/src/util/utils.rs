use crate::api::update;
use crate::state::db;
///
/// [AR] Feature Utils
///
/// Version: 0.1.1
///
use crate::{Result, State};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process;
use std::time::SystemTime;
use tokio::{fs, io};

const PACKAGE_JSON_CONTENT: &str =
    // include_str!("../../../../apps/app-frontend/package.json");
    include_str!("../../../../apps/app/tauri.conf.json");

#[derive(Serialize, Deserialize)]
pub struct Launcher {
    pub version: String,
}

#[derive(Debug, Deserialize)]
struct Artifact {
    path: Option<String>,
    sha1: Option<String>,
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Downloads {
    artifact: Option<Artifact>,
}

#[derive(Debug, Deserialize)]
struct Library {
    name: String,
    downloads: Option<Downloads>,
}

#[derive(Debug, Deserialize)]
struct VersionJson {
    libraries: Vec<Library>,
}

/// Deserialize the content of package.json into a Launcher struct
pub fn read_package_json() -> io::Result<Launcher> {
    let launcher: Launcher = serde_json::from_str(PACKAGE_JSON_CONTENT)?;
    Ok(launcher)
}

/// ### AR • Ely.by Injector
/// Returns the PathBuf to the Ely.by AuthLib Injector
/// If resource doesn't exist or outdated, it will be downloaded from Git Astralium.
pub async fn get_or_download_elyby_injector() -> Result<PathBuf> {
    tracing::info!("[AR] • Initializing state for Ely.by AuthLib Injector...");
    let state = State::get().await?;
    let libraries_dir = state.directories.libraries_dir();

    // Stores the local authlib injectors from `libraries/sqrilizz/authlib_injectors/` directory.
    let mut local_authlib_injectors = Vec::new();

    validate_sqrilizz_library_dir(&libraries_dir, "authlib_injector/").await?;
    let sqrilizz_dir = libraries_dir.join("sqrilizz/");
    let authlib_injector_dir = sqrilizz_dir.join("authlib_injector/");
    let mut authlib_injector_dir_data =
        fs::read_dir(&authlib_injector_dir).await?;

    // Get all local authlib injectors
    while let Some(entry) = authlib_injector_dir_data.next_entry().await? {
        let path = entry.path();
        if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
            if file_name.starts_with("authlib-injector") {
                let metadata = entry.metadata().await?;
                let modified =
                    metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                local_authlib_injectors.push((path.clone(), modified));
            }
        }
    }

    // Get information about latest authlib injector from remote repository
    let (asset_name, download_url) = match extract_elyby_authlib_metadata(
        "authlib-injector",
    )
    .await
    {
        Ok(data) => data,
        Err(err) => {
            if let Some((local_path, _)) =
                local_authlib_injectors.iter().max_by(|a, b| a.1.cmp(&b.1))
            {
                tracing::info!("[AR] • Found local AuthLib Injector(s):");
                for (path, time) in &local_authlib_injectors {
                    tracing::info!(
                        "• {:?} (modified: {:?})",
                        path.file_name().unwrap(),
                        time
                    );
                }
                tracing::warn!(
                    "[AR] • Failed to get latest AuthLib Injector from remote, using latest local version: {}",
                    local_path.display()
                );
                return Ok(local_path.clone());
            } else {
                tracing::error!(
                    "[AR] • Failed to get AuthLib Injector from remote and no local copy found."
                );
                return Err(crate::ErrorKind::NetworkErrorOccurred { error: format!("Failed to fetch authlib-injector metadata and no local version available: {}", err) }.as_error());
            }
        }
    };

    if !local_authlib_injectors.is_empty() {
        local_authlib_injectors.sort_by(|a, b| a.1.cmp(&b.1));
        tracing::info!("[AR] • Found local AuthLib Injector(s):");
        for (path, time) in &local_authlib_injectors {
            tracing::info!(
                "• {:?} (modified: {:?})",
                path.file_name().unwrap(),
                time
            );
        }
    }

    let remote_authlib_injector = if !asset_name.is_empty() {
        authlib_injector_dir.join(&asset_name)
    } else {
        return Err(crate::ErrorKind::ParseError {
            reason: "Asset name is empty from metadata".to_string(),
        }
        .as_error());
    };

    let latest_local_authlib_injector =
        local_authlib_injectors.first().map(|(p, _)| p.clone());

    let latest_local_authlib_injector_full_path_buf =
        match latest_local_authlib_injector {
            Some(path) => path,
            None => {
                tracing::info!(
                    "[AR] • No local version found, will download from remote: {}",
                    remote_authlib_injector.display()
                );
                let bytes = fetch_bytes_from_url(download_url.as_str()).await?;
                write_file_to_libraries(
                    &remote_authlib_injector.to_string_lossy(),
                    &bytes,
                )
                .await?;
                tracing::info!(
                    "[AR] • Successfully saved AuthLib Injector to {}",
                    remote_authlib_injector.display()
                );
                return Ok(remote_authlib_injector);
            }
        };

    tracing::info!("[AR] • Remote Asset name: {}", asset_name);
    tracing::info!("[AR] • Remote Download URL: {}", download_url);
    tracing::info!(
        "[AR] • Latest local AuthLib Injector: {}",
        latest_local_authlib_injector_full_path_buf
            .file_name()
            .unwrap()
            .to_string_lossy()
    );
    tracing::info!(
        "[AR] • Comparing local version {} with parsed remote version {}",
        latest_local_authlib_injector_full_path_buf.display(),
        remote_authlib_injector.display()
    );

    if remote_authlib_injector == latest_local_authlib_injector_full_path_buf {
        tracing::info!(
            "[AR] • Remote version is the same as local version, using local copy."
        );
        return Ok(latest_local_authlib_injector_full_path_buf);
    } else {
        tracing::info!(
            "[AR] • Doesn't exist or outdated, attempting to download latest AuthLib Injector from URL: {}",
            download_url
        );
        let bytes = fetch_bytes_from_url(download_url.as_str()).await?;
        write_file_to_libraries(
            &remote_authlib_injector.to_string_lossy(),
            &bytes,
        )
        .await?;
        tracing::info!(
            "[AR] • Successfully saved AuthLib Injector to {}",
            remote_authlib_injector.display()
        );
        return Ok(remote_authlib_injector);
    }
}

/// ### AR • Migration. Patch
/// Applying migration fix for SQLite database.
pub async fn apply_migration_fix(eol: &str) -> Result<bool> {
    tracing::info!("[AR] • Attempting to apply migration fix");
    let patched = db::apply_migration_fix(eol).await?;
    if patched {
        tracing::info!("[AR] • Successfully applied migration fix");
    } else {
        tracing::error!("[AR] • Failed to apply migration fix");
    }
    Ok(patched)
}

/// ### AR • Feature. Updater
/// Initialize the update launcher.
pub async fn init_update_launcher(
    download_url: &str,
    local_filename: &str,
    os_type: &str,
    auto_update_supported: bool,
) -> Result<()> {
    tracing::info!("[AR] • Initialize downloading from • {:?}", download_url);
    tracing::info!("[AR] • Save local file name • {:?}", local_filename);
    tracing::info!("[AR] • OS type • {}", os_type);
    tracing::info!("[AR] • Auto update supported • {}", auto_update_supported);

    if let Err(e) = update::get_resource(
        download_url,
        local_filename,
        os_type,
        auto_update_supported,
    )
    .await
    {
        eprintln!(
            "[AR] • An error occurred! Failed to download the file: {}",
            e
        );
    } else {
        println!("[AR] • Code finishes without errors.");
        process::exit(0)
    }
    Ok(())
}

/// ### AR • AuthLib (Ely.by)
/// Initializes the AuthLib patching process.
///
/// Returns `true` if the authlib patched successfully.
pub async fn init_authlib_patching(
    minecraft_version: &str,
    is_mojang: bool,
) -> Result<bool> {
    let minecraft_library_metadata =
        get_minecraft_library_metadata(minecraft_version).await?;
    // Parses the AuthLib version from string
    // Example output: "com.mojang:authlib:6.0.58" -> "6.0.58"
    let authlib_version = minecraft_library_metadata
        .name
        .split(':')
        .nth(2)
        .unwrap_or("unknown");
    let authlib_fullname_string = format!("authlib-{}.jar", authlib_version);
    let authlib_fullname_str = authlib_fullname_string.as_str();

    tracing::info!(
        "[AR] • Attempting to download AuthLib -> {}.",
        authlib_fullname_string
    );

    download_authlib(
        &minecraft_library_metadata,
        authlib_fullname_str,
        minecraft_version,
        is_mojang,
    )
    .await
}

/// ### SL • Universal Write (IO) Function.
/// Validating the `sqrilizz/{target_directory}/` directory exists inside the libraries/sqrilizz directory.
async fn validate_sqrilizz_library_dir(
    libraries_dir: &PathBuf,
    validation_directory: &str,
) -> Result<()> {
    let sqrilizz_path =
        libraries_dir.join(format!("sqrilizz/{}", validation_directory));
    if !sqrilizz_path.exists() {
        tokio::fs::create_dir_all(&sqrilizz_path)
            .await
            .map_err(|e| {
                tracing::error!(
                    "[SL] • Failed to create {} directory: {:?}",
                    sqrilizz_path.display(),
                    e
                );
                crate::ErrorKind::IOErrorOccurred {
                    error: format!(
                        "Failed to create {} directory: {}",
                        sqrilizz_path.display(),
                        e
                    ),
                }
                .as_error()
            })?;
        tracing::info!(
            "[SL] • Created missing {} directory",
            sqrilizz_path.display()
        );
    }
    Ok(())
}

/// ### AR • Universal Write (IO) Function
/// Saves the downloaded bytes to the `libraries` directory using the given relative path.
async fn write_file_to_libraries(
    relative_path: &str,
    bytes: &bytes::Bytes,
) -> Result<()> {
    let state = State::get().await?;
    let output_path = state.directories.libraries_dir().join(relative_path);

    fs::write(&output_path, bytes).await.map_err(|e| {
        tracing::error!("[AR] • Failed to save file: {:?}", e);
        crate::ErrorKind::IOErrorOccurred {
            error: format!("Failed to save file: {e}"),
        }
        .as_error()
    })
}

/// ### AR • AuthLib (Ely.by)
/// Downloads the AuthLib file from Mojang libraries or Git Astralium services.
async fn download_authlib(
    minecraft_library_metadata: &Library,
    authlib_fullname: &str,
    minecraft_version: &str,
    is_mojang: bool,
) -> Result<bool> {
    let state = State::get().await?;
    let (mut url, path) = extract_minecraft_local_download_info(
        minecraft_library_metadata,
        minecraft_version,
    )?;
    let full_path = state.directories.libraries_dir().join(path);

    if !is_mojang {
        tracing::info!(
            "[AR] • Attempting to download AuthLib from Git Astralium"
        );
        (_, url) = extract_elyby_authlib_metadata(authlib_fullname).await?;
    }
    tracing::info!("[AR] • Downloading AuthLib from URL: {}", url);
    let bytes = fetch_bytes_from_url(&url).await?;
    tracing::info!("[AR] • Will save to path: {}", full_path.to_str().unwrap());
    write_file_to_libraries(full_path.to_str().unwrap(), &bytes).await?;
    tracing::info!("[AR] • Successfully saved AuthLib to {:?}", full_path);
    Ok(true)
}

/// ### AR • AuthLib (Ely.by)
/// Parses the ElyIntegration release JSON and returns the download URL for the given AuthLib version.
async fn extract_elyby_authlib_metadata(
    authlib_fullname: &str,
) -> Result<(String, String)> {
    const URL: &str =
        "https://api.github.com/repos/sqrilizz/ElyIntegration/releases/latest";

    let response = reqwest::get(URL).await.map_err(|e| {
        tracing::error!(
            "[AR] • Failed to fetch ElyIntegration release JSON: {:?}",
            e
        );
        crate::ErrorKind::NetworkErrorOccurred {
            error: format!(
                "Failed to fetch ElyIntegration release JSON: {}",
                e
            ),
        }
        .as_error()
    })?;

    let json: serde_json::Value = response.json().await.map_err(|e| {
        tracing::error!("[AR] • Failed to parse ElyIntegration JSON: {:?}", e);
        crate::ErrorKind::ParseError {
            reason: format!("Failed to parse ElyIntegration JSON: {}", e),
        }
        .as_error()
    })?;

    let assets =
        json.get("assets")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                crate::ErrorKind::ParseError {
                    reason: "Missing 'assets' array".into(),
                }
                .as_error()
            })?;

    let asset = assets
        .iter()
        .find(|a| {
            a.get("name")
                .and_then(|n| n.as_str())
                .map(|n| n.contains(authlib_fullname))
                .unwrap_or(false)
        })
        .ok_or_else(|| {
            crate::ErrorKind::ParseError {
                reason: format!(
                    "No matching asset for {} in ElyIntegration JSON response.",
                    authlib_fullname
                ),
            }
            .as_error()
        })?;

    let download_url = asset
        .get("browser_download_url")
        .and_then(|u| u.as_str())
        .ok_or_else(|| {
            crate::ErrorKind::ParseError {
                reason: "Missing 'browser_download_url'".into(),
            }
            .as_error()
        })?
        .to_string();

    let asset_name = asset
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| {
            crate::ErrorKind::ParseError {
                reason: "Missing 'name'".into(),
            }
            .as_error()
        })?
        .to_string();

    Ok((asset_name, download_url))
}

/// ### AR • AuthLib (Ely.by)
/// Extracts the artifact URL and Path from the library structure.
///
/// Returns a tuple of references to the URL and path strings,
/// or an error if the required metadata is missing.
fn extract_minecraft_local_download_info(
    minecraft_library_metadata: &Library,
    minecraft_version: &str,
) -> Result<(String, String)> {
    let artifact = minecraft_library_metadata
        .downloads
        .as_ref()
        .and_then(|d| d.artifact.as_ref())
        .ok_or_else(|| {
            crate::ErrorKind::MinecraftMetadataNotFound {
                minecraft_version: minecraft_version.to_string(),
            }
            .as_error()
        })?;

    let url = artifact.url.clone().ok_or_else(|| {
        crate::ErrorKind::MinecraftMetadataNotFound {
            minecraft_version: minecraft_version.to_string(),
        }
        .as_error()
    })?;

    let path = artifact.path.clone().ok_or_else(|| {
        crate::ErrorKind::MinecraftMetadataNotFound {
            minecraft_version: minecraft_version.to_string(),
        }
        .as_error()
    })?;

    Ok((url, path))
}

/// ### AR • Universal Fetch Bytes (IO)
/// Downloads bytes from the provided URL with a 15 second timeout.
async fn fetch_bytes_from_url(url: &str) -> Result<bytes::Bytes> {
    // Create client instance with request timeout.
    let client = reqwest::Client::new();
    const TIMEOUT_SECONDS: u64 = 15;

    let response = tokio::time::timeout(
        std::time::Duration::from_secs(TIMEOUT_SECONDS),
        client.get(url).send(),
    )
    .await
    .map_err(|_| {
        tracing::error!(
            "[AR] • Download timed out after {} seconds",
            TIMEOUT_SECONDS
        );
        crate::ErrorKind::NetworkErrorOccurred {
            error: format!(
                "Download timed out after {TIMEOUT_SECONDS} seconds"
            )
            .to_string(),
        }
        .as_error()
    })?
    .map_err(|e| {
        tracing::error!("[AR] • Request error: {:?}", e);
        crate::ErrorKind::NetworkErrorOccurred {
            error: format!("Request error: {e}"),
        }
        .as_error()
    })?;

    if !response.status().is_success() {
        let status = response.status().to_string();
        tracing::error!("[AR] • Failed to download file: HTTP {}", status);
        return Err(crate::ErrorKind::NetworkErrorOccurred {
            error: format!("Failed to download file: HTTP {status}"),
        }
        .as_error());
    }

    response.bytes().await.map_err(|e| {
        tracing::error!("[AR] • Failed to read response bytes: {:?}", e);
        crate::ErrorKind::NetworkErrorOccurred {
            error: format!("Failed to read response bytes: {e}"),
        }
        .as_error()
    })
}

/// ### AR • AuthLib (Ely.by)
/// Gets the Minecraft library metadata from the local libraries directory.
async fn get_minecraft_library_metadata(
    minecraft_version: &str,
) -> Result<Library> {
    let state = State::get().await?;

    let path = state
        .directories
        .version_dir(minecraft_version)
        .join(format!("{}.json", minecraft_version));
    if !path.exists() {
        tracing::error!("[AR] • File not found: {:#?}", path);
        return Err(crate::ErrorKind::InvalidMinecraftVersion {
            minecraft_version: minecraft_version.to_string(),
        }
        .as_error());
    }

    let content = fs::read_to_string(&path).await?;
    let version_data: VersionJson = serde_json::from_str(&content)?;

    for lib in version_data.libraries {
        if lib.name.contains("com.mojang:authlib") {
            if let Some(downloads) = &lib.downloads {
                if let Some(artifact) = &downloads.artifact {
                    if artifact.path.is_some()
                        && artifact.url.is_some()
                        && artifact.sha1.is_some()
                    {
                        tracing::info!("[AR] • Found AuthLib: {}", lib.name);
                        tracing::info!(
                            "[AR] • Path: {}",
                            artifact.path.as_ref().unwrap()
                        );
                        tracing::info!(
                            "[AR] • URL: {}",
                            artifact.url.as_ref().unwrap()
                        );
                        tracing::info!(
                            "[AR] • SHA1: {}",
                            artifact.sha1.as_ref().unwrap()
                        );

                        return Ok(lib);
                    }
                }
            }
        }
    }

    Err(crate::ErrorKind::MinecraftMetadataNotFound {
        minecraft_version: minecraft_version.to_string(),
    }
    .as_error())
}
