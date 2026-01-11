#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![recursion_limit = "256"]

use native_dialog::{DialogBuilder, MessageLevel};
use std::env;
use tauri::{Listener, Manager};
use theseus::prelude::*;

mod api;
mod error;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(feature = "updater")]
mod updater_impl;
#[cfg(not(feature = "updater"))]
mod updater_impl_noop;

// Should be called in launcher initialization
#[tracing::instrument(skip_all)]
#[tauri::command]
async fn initialize_state(app: tauri::AppHandle) -> api::Result<()> {
    tracing::info!("Initializing app event state...");
    theseus::EventState::init(app.clone()).await?;
    tracing::info!("Initializing app state...");
    State::init().await?;
    tracing::info!("Sqrilizz Launcher state successfully initialized.");
    let state = State::get().await?;
    app.asset_protocol_scope()
        .allow_directory(state.directories.caches_dir(), true)?;
    app.asset_protocol_scope()
        .allow_directory(state.directories.caches_dir().join("icons"), true)?;

    // Check for existing Minecraft processes and restore session state
    tracing::info!("Checking for existing Minecraft processes...");
    let running_processes = process::get_all().await?;
    if !running_processes.is_empty() {
        tracing::info!(
            "Found {} running Minecraft process(es), session will be restored",
            running_processes.len()
        );
    }

    Ok(())
}

// Should be call once Vue has mounted the app
#[tracing::instrument(skip_all)]
#[tauri::command]
fn show_window(app: tauri::AppHandle) {
    let win = app.get_window("main").unwrap();
    if let Err(e) = win.show() {
        DialogBuilder::message()
            .set_level(MessageLevel::Error)
            .set_title("Initialization error")
            .set_text(format!(
                "Cannot display application window due to an error:\n{e}"
            ))
            .alert()
            .show()
            .unwrap();
        panic!("cannot display application window")
    } else {
        let _ = win.set_focus();
        let _ = win.unminimize();

        #[cfg(target_os = "linux")]
        {
            // Additional Linux-specific window activation
            let _ = win.set_always_on_top(true);
            std::thread::sleep(std::time::Duration::from_millis(50));
            let _ = win.set_always_on_top(false);
        }
    }
}

// [AR] Write news cache to file
#[tauri::command]
async fn write_news_cache(content: String) -> api::Result<()> {
    use tokio::fs;

    let state = State::get().await?;
    let cache_dir = state.directories.caches_dir();
    let news_file = cache_dir.join("news.json");

    // Ensure cache directory exists
    fs::create_dir_all(&cache_dir).await?;

    // Write news content
    fs::write(&news_file, content).await?;

    tracing::info!("News cache written to {:?}", news_file);
    Ok(())
}

// [AR] Read news cache from file
#[tauri::command]
async fn read_news_cache() -> api::Result<String> {
    use tokio::fs;

    let state = State::get().await?;
    let cache_dir = state.directories.caches_dir();
    let news_file = cache_dir.join("news.json");

    // Try to read news cache
    match fs::read_to_string(&news_file).await {
        Ok(content) => {
            tracing::info!("News cache read from {:?}", news_file);
            Ok(content)
        }
        Err(e) => {
            tracing::warn!("Failed to read news cache: {}", e);
            // Return empty articles if file doesn't exist
            Ok(r#"{"articles":[]}"#.to_string())
        }
    }
}

// [AR] Fetch news from API (bypasses CSP)
#[tauri::command]
async fn fetch_news_from_api() -> api::Result<String> {
    use theseus::util::fetch::REQWEST_CLIENT;

    let url = "http://151.241.155.196:3000";
    tracing::info!("Fetching news from API: {}", url);

    let response = REQWEST_CLIENT
        .get(url)
        .header("X-API-Key", "SqrilizzTestKey")
        .send()
        .await
        .map_err(|e| {
            theseus::Error::from(theseus::ErrorKind::OtherError(format!(
                "Failed to fetch news: {}",
                e
            )))
        })?;

    let text = response.text().await.map_err(|e| {
        theseus::Error::from(theseus::ErrorKind::OtherError(format!(
            "Failed to read response: {}",
            e
        )))
    })?;

    tracing::info!("News fetched successfully from API");
    Ok(text)
}

#[tauri::command]
fn is_dev() -> bool {
    cfg!(debug_assertions)
}

#[tauri::command]
fn are_updates_enabled() -> bool {
    cfg!(feature = "updater")
        && env::var("MODRINTH_EXTERNAL_UPDATE_PROVIDER").is_err()
}

#[cfg(feature = "updater")]
pub use updater_impl::*;

#[cfg(not(feature = "updater"))]
pub use updater_impl_noop::*;

// Toggles decorations
#[tauri::command]
async fn toggle_decorations(b: bool, window: tauri::Window) -> api::Result<()> {
    window.set_decorations(b).map_err(|e| {
        theseus::Error::from(theseus::ErrorKind::OtherError(format!(
            "Failed to toggle decorations: {e}"
        )))
    })?;
    Ok(())
}

#[tauri::command]
fn restart_app(app: tauri::AppHandle) {
    app.restart();
}

// Check and restore running Minecraft sessions
#[tauri::command]
async fn check_running_sessions() -> api::Result<Vec<ProcessMetadata>> {
    let processes = process::get_all().await?;
    tracing::info!("Found {} running Minecraft session(s)", processes.len());
    Ok(processes)
}

// if Tauri app is called with arguments, then those arguments will be treated as commands
// ie: deep links or filepaths for .mrpacks
fn main() {
    /*
        tracing is set basd on the environment variable RUST_LOG=xxx, depending on the amount of logs to show
            ERROR > WARN > INFO > DEBUG > TRACE
        eg. RUST_LOG=info will show info, warn, and error logs
            RUST_LOG="theseus=trace" will show *all* messages but from theseus only (and not dependencies using similar crates)
            RUST_LOG="theseus=trace" will show *all* messages but from theseus only (and not dependencies using similar crates)

        Error messages returned to Tauri will display as traced error logs if they return an error.
        This will also include an attached span trace if the error is from a tracing error, and the level is set to info, debug, or trace

        on unix:
            RUST_LOG="theseus=trace" {run command}

    */
    let _log_guard = theseus::start_logger();

    tracing::info!(
        "Initialized tracing subscriber. Loading Sqrilizz Launcher!"
    );

    let mut builder = tauri::Builder::default();

    builder = builder
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if let Some(payload) = args.get(1) {
                tracing::info!("Handling deep link from arg {payload}");
                let payload = payload.clone();
                tauri::async_runtime::spawn(api::utils::handle_command(
                    payload,
                ));
            }

            if let Some(win) = app.get_window("main") {
                let _ = win.set_focus();
            }
        }))
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        // [AR] Patch: Disabled window-state plugin to allow window resizing
        // .plugin(
        //     tauri_plugin_window_state::Builder::default()
        //         .with_filename(
        //             if std::env::current_dir()
        //                 .ok()
        //                 .map(|dir| dir.join("portable.txt").exists())
        //                 .unwrap_or(false)
        //             {
        //                 std::env::current_dir()
        //                     .ok()
        //                     .map(|dir| {
        //                         dir.join("UserData/app-window-state.json")
        //                             .to_string_lossy()
        //                             .into_owned()
        //                     })
        //                     .unwrap()
        //             } else {
        //                 "app-window-state.json".to_string()
        //             },
        //         )
        //         .with_state_flags(
        //             tauri_plugin_window_state::StateFlags::POSITION
        //                 | tauri_plugin_window_state::StateFlags::MAXIMIZED,
        //         )
        //         .build(),
        // )
        .setup(|app| {
            tracing::info!("Running setup hook...");

            #[cfg(target_os = "macos")]
            {
                let payload = macos::deep_link::get_or_init_payload(app);

                let mtx_copy = payload.payload;
                app.listen("deep-link://new-url", move |url| {
                    let mtx_copy_copy = mtx_copy.clone();
                    let request = url.payload().to_owned();

                    let actual_request =
                        serde_json::from_str::<Vec<String>>(&request)
                            .ok()
                            .map(|mut x| x.remove(0))
                            .unwrap_or(request);

                    tauri::async_runtime::spawn(async move {
                        tracing::info!("Handling deep link {actual_request}");

                        let mut payload = mtx_copy_copy.lock().await;
                        if payload.is_none() {
                            *payload = Some(actual_request.clone());
                        }

                        let _ =
                            api::utils::handle_command(actual_request).await;
                    });
                });
            };

            #[cfg(not(target_os = "macos"))]
            app.listen("deep-link://new-url", |url| {
                let payload = url.payload().to_owned();
                tracing::info!("Handling deep link {payload}");
                tauri::async_runtime::spawn(api::utils::handle_command(
                    payload,
                ));
            });

            #[cfg(not(target_os = "linux"))]
            if let Some(window) = app.get_window("main")
                && let Err(e) = window.set_shadow(true)
            {
                tracing::warn!("Failed to set window shadow: {e}");
            }

            tracing::info!("Setup hook completed successfully");

            // Ensure window is visible and focused (especially important for Linux)
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.unminimize();

                #[cfg(target_os = "linux")]
                {
                    // Extra measures for Linux to ensure window appears
                    let _ = window.set_always_on_top(true);
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    let _ = window.set_always_on_top(false);
                }

                tracing::info!("Window show() and focus() called explicitly");
            }

            Ok(())
        });

    builder = builder
        .plugin(api::auth::init())
        // [AR] Patch: Removed Modrinth auth plugin
        .plugin(api::import::init())
        .plugin(api::logs::init())
        .plugin(api::jre::init())
        .plugin(api::metadata::init())
        // [AR] Patch: REMOVED minecraft_skins plugin - causes crashes
        .plugin(api::pack::init())
        .plugin(api::process::init())
        .plugin(api::profile::init())
        .plugin(api::profile_create::init())
        .plugin(api::settings::init())
        .plugin(api::tags::init())
        .plugin(api::utils::init())
        .plugin(api::cache::init())
        // [AR] Patch: Removed friends plugin
        .plugin(api::worlds::init())
        .manage(PendingUpdateData::default())
        .invoke_handler(tauri::generate_handler![
            initialize_state,
            is_dev,
            are_updates_enabled,
            get_update_size,
            enqueue_update_for_installation,
            remove_enqueued_update,
            toggle_decorations,
            show_window,
            write_news_cache,
            read_news_cache,
            fetch_news_from_api,
            restart_app,
            check_running_sessions,
        ]);

    tracing::info!("Initializing app...");
    let app = builder.build(tauri::generate_context!());

    tracing::info!("App built successfully, entering event loop...");

    match app {
        Ok(app) => {
            tracing::info!("Launcher entering persistent GUI mode");

            app.run(|app, event| {
                // Log important events for debugging
                match &event {
                    tauri::RunEvent::Exit => {
                        tracing::info!("Received Exit event");
                    }
                    tauri::RunEvent::ExitRequested { .. } => {
                        tracing::info!("Received ExitRequested event, allowing exit");
                        // [AR] Patch: Removed api.prevent_exit() to allow proper window closing
                    }
                    tauri::RunEvent::WindowEvent { label, event, .. } => {
                        match event {
                            tauri::WindowEvent::CloseRequested { .. } => {
                                tracing::info!("Window '{}' close requested", label);
                            }
                            tauri::WindowEvent::Destroyed => {
                                tracing::info!("Window '{}' destroyed", label);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }

                #[cfg(not(any(feature = "updater", target_os = "macos")))]
                drop((app, event));

                #[cfg(feature = "updater")]
                if matches!(event, tauri::RunEvent::Exit) {
                    let update_data = app.state::<PendingUpdateData>().inner();
                    if let Some((update, data)) = &*update_data.0.lock().unwrap() {
                        fn set_changelog_toast(version: Option<String>) {
                            let toast_result: theseus::Result<()> = tauri::async_runtime::block_on(async move {
                                let mut settings = settings::get().await?;
                                settings.pending_update_toast_for_version = version;
                                settings::set(settings).await?;
                                Ok(())
                            });
                            if let Err(e) = toast_result {
                                tracing::warn!("Failed to set pending_update_toast: {e}")
                            }
                        }

                        set_changelog_toast(Some(update.version.clone()));
                        if let Err(e) = update.install(data) {
                            tracing::error!("Error while updating: {e}");
                            set_changelog_toast(None);

                            DialogBuilder::message()
                                .set_level(MessageLevel::Error)
                                .set_title("Update error")
                                .set_text(format!("Failed to install update due to an error:\n{e}"))
                                .alert()
                                .show()
                                .unwrap();
                        }
                        app.restart();
                    }
                }

                #[cfg(target_os = "macos")]
                if let tauri::RunEvent::Opened { urls } = event {
                    tracing::info!("Handling webview open {urls:?}");

                    let file = urls
                        .into_iter()
                        .find_map(|url| url.to_file_path().ok());

                    if let Some(file) = file {
                        let payload =
                            macos::deep_link::get_or_init_payload(app);

                        let mtx_copy = payload.payload;
                        let request = file.to_string_lossy().to_string();
                        tauri::async_runtime::spawn(async move {
                            let mut payload = mtx_copy.lock().await;
                            if payload.is_none() {
                                *payload = Some(request.clone());
                            }

                            let _ = api::utils::handle_command(request).await;
                        });
                    }
                }
            });
        }
        Err(e) => {
            tracing::error!("Error while running tauri application: {:?}", e);

            #[cfg(target_os = "windows")]
            {
                // tauri doesn't expose runtime errors, so matching a string representation seems like the only solution
                if format!("{e:?}").contains(
                    "Runtime(CreateWebview(WebView2Error(WindowsError",
                ) {
                    DialogBuilder::message()
                        .set_level(MessageLevel::Error)
                        .set_title("Initialization error")
                        .set_text("Your Microsoft Edge WebView2 installation is corrupt.\n\nMicrosoft Edge WebView2 is required to run Modrinth App.\n\nLearn how to repair it at https://support.modrinth.com/en/articles/8797765-corrupted-microsoft-edge-webview2-installation")
                        .alert()
                        .show()
                        .unwrap();

                    panic!("webview2 initialization failed")
                }
            }

            DialogBuilder::message()
                .set_level(MessageLevel::Error)
                .set_title("Initialization error")
                .set_text(format!(
                    "Cannot initialize application due to an error:\n{e:?}"
                ))
                .alert()
                .show()
                .unwrap();

            panic!("{1}: {:?}", e, "error while running tauri application")
        }
    }
}
