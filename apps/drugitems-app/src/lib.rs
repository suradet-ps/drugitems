//! DrugItems desktop shell (Tauri 2 backend).

pub mod commands;
pub mod report;
pub mod state;

use state::AppState;
use tauri::Manager;

/// Start the Tauri application.
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "drugitems_app=info,drugitems_db=info,drugitems_config=info".into()
            }),
        )
        .init();

    tauri::Builder::default()
        .manage(AppState::new())
        .setup(|app| {
            // Re-center explicitly after creation: the `center` config is
            // applied while the window is still being created, and on
            // Windows the OS can reposition/clamp the window afterwards
            // (especially in dev mode or on screens smaller than the
            // window). Shrink the window to the primary monitor's work area
            // first, then center - an oversized window gets clamped by the
            // OS, which throws the centering off.
            if let Some(window) = app.get_webview_window("main") {
                // The window starts hidden (visible: false in the config);
                // position it fully before revealing it, so the OS can never
                // show it mid-layout at the wrong spot.
                if let Some(monitor) = app.primary_monitor()? {
                    let size = window.outer_size()?;
                    let scale = monitor.scale_factor();
                    let work = monitor.work_area();
                    let work_w = work.size.width as f64 / scale;
                    let work_h = work.size.height as f64 / scale;
                    let width = (size.width as f64).min(work_w);
                    let height = (size.height as f64).min(work_h);
                    if width < size.width as f64 || height < size.height as f64 {
                        window.set_size(tauri::LogicalSize::new(width, height))?;
                    }
                }
                window.center()?;
                window.show()?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_status,
            commands::is_configured,
            commands::connection_health,
            commands::save_connection,
            commands::get_connection,
            commands::test_connection,
            commands::clear_site_config,
            commands::get_settings,
            commands::save_settings,
            commands::choose_snapshot,
            commands::snapshot_info,
            commands::run_compare,
            commands::export_report,
        ])
        .run(tauri::generate_context!())
        .expect("error while running DrugItems");
}
