use crate::api::Result;
use chrono::{Duration, Utc};
use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime, UserAttentionType};
use tauri_plugin_http::reqwest::Client;
use theseus::prelude::*;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::<R>::new("auth")
        .invoke_handler(tauri::generate_handler![
            offline_login,
            elyby_login,
            elyby_auth_authenticate,
            check_reachable,
            login,
            remove_user,
            get_default_user,
            set_default_user,
            get_users,
        ])
        .build()
}

/// ### AR • Feature
/// Create new offline user
#[tauri::command]
pub async fn offline_login(name: &str) -> Result<Credentials> {
    let credentials = minecraft_auth::offline_auth(name).await?;
    Ok(credentials)
}

/// ### AR • Feature
/// Create new Ely.by user
#[tauri::command]
pub async fn elyby_login(
    uuid: uuid::Uuid,
    login: &str,
    access_token: &str,
) -> Result<Credentials> {
    let credentials =
        minecraft_auth::elyby_auth(uuid, login, access_token).await?;
    Ok(credentials)
}

/// ### AR • Feature
/// Authenticate Ely.by user
#[tauri::command]
pub async fn elyby_auth_authenticate(
    login: &str,
    password: &str,
    client_token: &str,
) -> Result<String> {
    let client = Client::new();
    let auth_body = serde_json::json!({
        "username": login,
        "password": password,
        "clientToken": client_token,
    });

    let response = match client
        .post("https://authserver.ely.by/auth/authenticate")
        .header("Content-Type", "application/json")
        .json(&auth_body)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            tracing::error!("[AR] • Failed to send request: {}", e);
            return Ok("".to_string());
        }
    };

    let text = match response.text().await {
        Ok(body) => body,
        Err(e) => {
            tracing::error!("[AR] • Failed to read response text: {}", e);
            return Ok("".to_string());
        }
    };
    Ok(text)
}

/// Checks if the authentication servers are reachable.
#[tauri::command]
pub async fn check_reachable() -> Result<()> {
    minecraft_auth::check_reachable().await?;
    Ok(())
}

/// Authenticate a user with Hydra - part 1
/// This begins the authentication flow quasi-synchronously, returning a URL to visit (that the user will sign in at)
#[tauri::command]
pub async fn login<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Option<Credentials>> {
    let flow = minecraft_auth::begin_login().await?;

    let start = Utc::now();

    if let Some(window) = app.get_webview_window("signin") {
        window.close()?;
    }

    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "signin",
        tauri::WebviewUrl::External(flow.auth_request_uri.parse().map_err(
            |_| {
                theseus::ErrorKind::OtherError(
                    "Error parsing auth redirect URL".to_string(),
                )
                .as_error()
            },
        )?),
    )
    .title("Sign into Modrinth")
    .always_on_top(true)
    .center()
    .build()?;

    window.request_user_attention(Some(UserAttentionType::Critical))?;

    while (Utc::now() - start) < Duration::minutes(10) {
        if window.title().is_err() {
            // user closed window, cancelling flow
            return Ok(None);
        }

        if window
            .url()?
            .as_str()
            .starts_with("https://login.live.com/oauth20_desktop.srf")
            && let Some((_, code)) =
                window.url()?.query_pairs().find(|x| x.0 == "code")
        {
            window.close()?;
            let val = minecraft_auth::finish_login(&code.clone(), flow).await?;

            return Ok(Some(val));
        }

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    window.close()?;
    Ok(None)
}

#[tauri::command]
pub async fn remove_user(user: uuid::Uuid) -> Result<()> {
    Ok(minecraft_auth::remove_user(user).await?)
}

#[tauri::command]
pub async fn get_default_user() -> Result<Option<uuid::Uuid>> {
    Ok(minecraft_auth::get_default_user().await?)
}

#[tauri::command]
pub async fn set_default_user(user: uuid::Uuid) -> Result<()> {
    Ok(minecraft_auth::set_default_user(user).await?)
}

/// Get a copy of the list of all user credentials
#[tauri::command]
pub async fn get_users() -> Result<Vec<Credentials>> {
    Ok(minecraft_auth::users().await?)
}
