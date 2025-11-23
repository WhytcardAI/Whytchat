# Project Debug Rules (Non-Obvious Only)

## Logging
- **Rust**: Uses `env_logger`. Logs appear in the terminal running `npm run tauri dev`.
  - Levels: `info!` for general flow, `error!` for recoverable errors.
  - Check `Supervisor` logs to trace message flow through actors.
- **Frontend**: Standard browser console.
  - **Tauri IPC Errors**: Often silent in UI if not caught. Wrap invokes in try/catch.

## File System Debugging
- **Path Resolution**: `PortablePathManager` behaves differently in Debug vs Release.
  - **Debug**: Uses workspace root + `apps/core/data` (usually).
  - **Release**: Uses executable directory + `data`.
  - **Gotcha**: If files seem "missing" after a build, check if you are looking in the release folder, not the source folder.

## Database Inspection
- **Location**: `./apps/core/data/db/whytchat.sqlite` (Dev).
- **Tool**: Use `sqlite3` CLI or a GUI tool to inspect data state if Actor behavior is confusing.
- **Locking**: SQLite writes lock the DB. Ensure `tokio` tasks yield and don't hold connections unnecessarily (use the pool).

## Actor Deadlocks
- **Symptoms**: UI hangs on "Thinking...", no logs.
- **Cause**: Circular channel dependencies or `recv().await` on a channel where the sender was dropped without sending.
- **Fix**: Check `process_message` logic in `Supervisor` and ensure `responder` is always called, even on error.