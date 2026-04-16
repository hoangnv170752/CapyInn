pub mod auth;
pub mod models;
pub mod proxy;
pub mod server;
pub mod tools;

use sqlx::{Pool, Sqlite};
use tauri::AppHandle;

use crate::app_identity;

/// Start the MCP Gateway SSE server on a background Tokio task.
/// Returns the port number the server is listening on.
pub async fn start_gateway(pool: Pool<Sqlite>, app_handle: AppHandle) -> Result<u16, String> {
    let port = server::start_server(pool, app_handle).await?;

    // Write port to lockfile
    if let Some(lockfile) = app_identity::gateway_lockfile_opt() {
        if let Some(parent) = lockfile.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create lockfile directory: {}", e))?;
        }
        std::fs::write(&lockfile, port.to_string())
            .map_err(|e| format!("Failed to write lockfile: {}", e))?;
    }

    eprintln!("MCP Gateway ready on :{}", port);
    Ok(port)
}

/// Clean up the lockfile on shutdown
pub fn cleanup_lockfile() {
    if let Some(lockfile) = app_identity::gateway_lockfile_opt() {
        let _ = std::fs::remove_file(&lockfile);
    }
}
