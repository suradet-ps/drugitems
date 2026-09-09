//! DrugItems desktop shell (Tauri 2 backend).

pub mod commands;
pub mod report;
pub mod state;

use state::AppState;

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
