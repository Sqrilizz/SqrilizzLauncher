//! Authentication flow interface

use reqwest::StatusCode;

use crate::State;
use crate::state::{Credentials, MinecraftLoginFlow};
use crate::util::fetch::REQWEST_CLIENT;

#[tracing::instrument]
pub async fn check_reachable() -> crate::Result<()> {
    let resp = REQWEST_CLIENT
        .get("https://sessionserver.mojang.com/session/minecraft/hasJoined")
        .send()
        .await?;
    if resp.status() == StatusCode::NO_CONTENT {
        return Ok(());
    }
    resp.error_for_status()?;
    Ok(())
}

#[tracing::instrument]
pub async fn begin_login() -> crate::Result<MinecraftLoginFlow> {
    let state = State::get().await?;

    crate::state::login_begin(&state.pool).await
}

#[tracing::instrument]
pub async fn finish_login(
    code: &str,
    flow: MinecraftLoginFlow,
) -> crate::Result<Credentials> {
    let state = State::get().await?;

    crate::state::login_finish(code, flow, &state.pool).await
}

#[tracing::instrument]
pub async fn offline_auth(name: &str) -> crate::Result<Credentials> {
    let state = State::get().await?;
    crate::state::offline_auth(name, &state.pool).await
}

#[tracing::instrument]
pub async fn elyby_auth(
    uuid: uuid::Uuid,
    login: &str,
    access_token: &str,
) -> crate::Result<Credentials> {
    let state = State::get().await?;
    crate::state::elyby_auth(uuid, login, access_token, &state.pool).await
}

#[tracing::instrument]
pub async fn get_default_user() -> crate::Result<Option<uuid::Uuid>> {
    let state = State::get().await?;
    let user = Credentials::get_active(&state.pool).await?;
    Ok(user.map(|user| user.offline_profile.id))
}

#[tracing::instrument]
pub async fn set_default_user(user: uuid::Uuid) -> crate::Result<()> {
    let state = State::get().await?;
    let users = Credentials::get_all(&state.pool).await?;

    // Check if user exists before trying to set as default
    if !users.contains_key(&user) {
        tracing::warn!(
            "Tried to set nonexistent user {} as default, ignoring",
            user
        );
        return Ok(());
    }

    // Collect IDs of other active users to deactivate
    let to_deactivate: Vec<uuid::Uuid> = users
        .iter()
        .filter(|entry| {
            let id = *entry.key();
            id != user && entry.value().active
        })
        .map(|entry| *entry.key())
        .collect();

    // Deactivate other users
    for id in to_deactivate {
        if let Some((_, mut creds)) = users.remove(&id) {
            creds.active = false;
            creds.upsert(&state.pool).await?;
        }
    }

    // Then activate the requested user
    let (_, mut user) = users.remove(&user).ok_or_else(|| {
        crate::ErrorKind::OtherError(format!(
            "Tried to get nonexistent user with ID {user}"
        ))
        .as_error()
    })?;

    user.active = true;
    user.upsert(&state.pool).await?;

    Ok(())
}

/// Remove a user account from the database
#[tracing::instrument]
pub async fn remove_user(uuid: uuid::Uuid) -> crate::Result<()> {
    let state = State::get().await?;

    let users = Credentials::get_all(&state.pool).await?;

    if let Some((uuid, user)) = users.remove(&uuid) {
        Credentials::remove(uuid, &state.pool).await?;

        if user.active
            && let Some((_, mut user)) = users.into_iter().next()
        {
            user.active = true;
            user.upsert(&state.pool).await?;
        }
    }

    Ok(())
}

/// Get a copy of the list of all user credentials
#[tracing::instrument]
pub async fn users() -> crate::Result<Vec<Credentials>> {
    let state = State::get().await?;
    let users = Credentials::get_all(&state.pool).await?;
    Ok(users.into_iter().map(|x| x.1).collect())
}
