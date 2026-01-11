use crate::ErrorKind;
use crate::state::DirectoryInfo;
use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions,
};
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::Instant;

pub(crate) async fn connect() -> crate::Result<Pool<Sqlite>> {
    let pool = connect_without_migrate().await?;

    sqlx::migrate!().run(&pool).await?;

    if let Err(err) = stale_data_cleanup(&pool).await {
        tracing::warn!(
            "Failed to clean up stale data from state database: {err}"
        );
    }

    Ok(pool)
}

// [AR] Feature. Implement SQLite3 connection without SQLx migrations.
async fn connect_without_migrate() -> crate::Result<Pool<Sqlite>> {
    let settings_dir = DirectoryInfo::get_initial_settings_dir().ok_or(
        ErrorKind::FSError("Could not find valid config dir".to_string()),
    )?;

    if !settings_dir.exists() {
        crate::util::io::create_dir_all(&settings_dir).await?;
    }

    let db_path = settings_dir.join("app.db");

    let uri = format!("sqlite:{}", db_path.display());
    let conn_options = SqliteConnectOptions::from_str(&uri)?
        .busy_timeout(Duration::from_secs(30))
        .journal_mode(SqliteJournalMode::Wal)
        .optimize_on_close(true, None)
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(100)
        .connect_with(conn_options)
        .await?;

    Ok(pool)
}

/// Cleans up data from the database that is no longer referenced, but must be
/// kept around for a little while to allow users to recover from accidental
/// deletions.
async fn stale_data_cleanup(pool: &Pool<Sqlite>) -> crate::Result<()> {
    let mut tx = pool.begin().await?;

    sqlx::query!(
        "DELETE FROM default_minecraft_capes WHERE minecraft_user_uuid NOT IN (SELECT uuid FROM minecraft_users)"
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "DELETE FROM custom_minecraft_skins WHERE minecraft_user_uuid NOT IN (SELECT uuid FROM minecraft_users)"
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}
/*
// [AR] Patch fix
Problem files, view detailed information in .gitattributes:
/packages/app-lib/migrations/20240711194701_init.sql !eol
CRLF -> 4c47e326f16f2b1efca548076ce638d4c90dd610172fe48c47d6de9bc46ef1c5abeadfdea05041ddd72c3819fa10c040
LF -> e973512979feac07e415405291eefafc1ef0bd89454958ad66f5452c381db8679c20ffadab55194ecf6ba8ec4ca2db21
/packages/app-lib/migrations/20240813205023_drop-active-unique.sql !eol
CRLF -> C8FD2EFE72E66E394732599EA8D93CE1ED337F098697B3ADAD40DD37CC6367893E199A8D7113B44A3D0FFB537692F91D
LF -> 5b53534a7ffd74eebede234222be47e1d37bd0cc5fee4475212491b0c0379c16e3079e08eee0af959b1fa20835eeb206
/packages/app-lib/migrations/20240930001852_disable-personalized-ads.sql !eol
CRLF -> C0DE804F171B5530010EDAE087A6E75645C0E90177E28365F935C9FDD9A5C68E24850B8C1498E386A379D525D520BC57
LF -> c0de804f171b5530010edae087a6e75645c0e90177e28365f935c9fdd9a5c68e24850b8c1498e386a379d525d520bc57
/packages/app-lib/migrations/20241222013857_feature-flags.sql !eol
CRLF -> 6B6F097E5BB45A397C96C3F1DC9C2A18433564E81DB264FE08A4775198CCEAC03C9E63C3605994ECB19C281C37D8F6AE
LF -> c17542cb989a0466153e695bfa4717f8970feee185ca186a2caa1f2f6c5d4adb990ab97c26cacfbbe09c39ac81551704
*/
pub(crate) async fn apply_migration_fix(eol: &str) -> crate::Result<bool> {
    let started = Instant::now();

    // Create connection to the database without migrations
    let pool = connect_without_migrate().await?;
    tracing::info!(
        "⚙️  Patching Modrinth corrupted migration checksums using EOL standard: {eol}"
    );

    // validate EOL input
    if eol != "lf" && eol != "crlf" {
        return Ok(false);
    }

    // [eol][version] -> checksum
    let checksums: HashMap<(&str, &str), &str> = HashMap::from([
        (
            ("lf", "20240711194701"),
            "e973512979feac07e415405291eefafc1ef0bd89454958ad66f5452c381db8679c20ffadab55194ecf6ba8ec4ca2db21",
        ),
        (
            ("crlf", "20240711194701"),
            "4c47e326f16f2b1efca548076ce638d4c90dd610172fe48c47d6de9bc46ef1c5abeadfdea05041ddd72c3819fa10c040",
        ),
        (
            ("lf", "20240813205023"),
            "5b53534a7ffd74eebede234222be47e1d37bd0cc5fee4475212491b0c0379c16e3079e08eee0af959b1fa20835eeb206",
        ),
        (
            ("crlf", "20240813205023"),
            "C8FD2EFE72E66E394732599EA8D93CE1ED337F098697B3ADAD40DD37CC6367893E199A8D7113B44A3D0FFB537692F91D",
        ),
        (
            ("lf", "20240930001852"),
            "c0de804f171b5530010edae087a6e75645c0e90177e28365f935c9fdd9a5c68e24850b8c1498e386a379d525d520bc57",
        ),
        (
            ("crlf", "20240930001852"),
            "C0DE804F171B5530010EDAE087A6E75645C0E90177E28365F935C9FDD9A5C68E24850B8C1498E386A379D525D520BC57",
        ),
        (
            ("lf", "20241222013857"),
            "c17542cb989a0466153e695bfa4717f8970feee185ca186a2caa1f2f6c5d4adb990ab97c26cacfbbe09c39ac81551704",
        ),
        (
            ("crlf", "20241222013857"),
            "6B6F097E5BB45A397C96C3F1DC9C2A18433564E81DB264FE08A4775198CCEAC03C9E63C3605994ECB19C281C37D8F6AE",
        ),
    ]);

    let mut changed = false;

    for ((eol_key, version), checksum) in checksums.iter() {
        if *eol_key != eol {
            continue;
        }

        tracing::info!(
            "⏳ Patching checksum for migration {version} ({})",
            eol.to_uppercase()
        );

        let result = sqlx::query(&format!(
            r#"
            UPDATE "_sqlx_migrations"
            SET checksum = X'{checksum}'
            WHERE version = '{version}';
            "#
        ))
        .execute(&pool)
        .await?;

        if result.rows_affected() > 0 {
            changed = true;
        }
    }

    tracing::info!(
        "✅ Checksum patching completed in {:.2?} (changes: {})",
        started.elapsed(),
        changed
    );

    Ok(changed)
}
