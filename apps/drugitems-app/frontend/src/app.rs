//! DrugItems frontend library (Leptos 0.8, CSR).
//!
//! Two-column desktop layout: left = snapshot + compare controls, right =
//! the results dashboard. On launch the app checks for stored connection
//! settings; if absent, an inline setup screen replaces the dashboard. The
//! top-bar status dot is driven by polling the backend's live
//! `connection_health`.

use std::time::Duration;

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

use crate::components::connection_form::ConnectionForm;
use crate::components::help_modal::HelpModal;
use crate::components::results::Results;
use crate::components::settings_modal::SettingsModal;
use crate::components::snapshot_panel::SnapshotPanel;
use crate::components::top_bar::TopBar;
use crate::state::{AppState, ConnectionHealth};

/// How often the frontend polls the backend's live health state.
const HEALTH_POLL_INTERVAL: Duration = Duration::from_secs(30);

/// Mounts the app into the document body.
pub fn run() {
    leptos::mount::mount_to_body(|| view! { <App /> });
}

/// Application shell.
#[component]
fn App() -> impl IntoView {
    let state = AppState::new();

    // First-run check + settings/snapshot mirroring.
    spawn_local(async move {
        let _ = crate::api::is_configured().await.map(|configured| {
            state.configured.set(configured);
        });
        if let Ok(status) = crate::api::get_app_status().await {
            state.site_label.set(status.site_label.unwrap_or_default());
            state.table_name.set(status.table_name);
        }
        if let Ok(settings) = crate::api::get_settings().await {
            if !settings.site_label.is_empty() {
                state.site_label.set(settings.site_label);
            }
            if !settings.table_name.is_empty() {
                state.table_name.set(settings.table_name);
            }
            state.ignore_columns.set(settings.ignore_columns);
        }
        if let Ok(meta) = crate::api::snapshot_info().await {
            state.snapshot.set(meta);
        }
    });

    // Poll the backend's live reachability - the status dot must reflect a
    // dead database within seconds, not "config exists".
    let poll_state = state;
    spawn_local(async move {
        loop {
            match crate::api::connection_health().await {
                Ok(health) => poll_state.health.set(health),
                Err(_) => poll_state.health.set(ConnectionHealth::Disconnected),
            }
            let delay = js_sys::Promise::new(&mut |resolve, _reject| {
                let f: &js_sys::Function = resolve.unchecked_ref();
                let _ = web_sys::window()
                    .expect("invariant: Tauri webview window")
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        f,
                        HEALTH_POLL_INTERVAL.as_millis() as i32,
                    );
            });
            wasm_bindgen_futures::JsFuture::from(delay)
                .await
                .expect("invariant: setTimeout promise resolves");
        }
    });

    view! {
        <div class="app">
            <TopBar state=state />
            <div class="app__body">
                {move || {
                    if state.configured.get() {
                        view! {
                            <div class="layout">
                                <aside class="sidebar">
                                    <SnapshotPanel state=state />
                                </aside>
                                <main class="main-canvas">
                                    <Results state=state />
                                </main>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <StartView state=state />
                        }.into_any()
                    }
                }}
            </div>
            <SettingsModal state=state />
            <HelpModal state=state />
        </div>
    }
}

/// First-run setup screen shown until a connection is configured.
#[component]
fn StartView(state: AppState) -> impl IntoView {
    view! {
        <div class="start">
            <div class="start__card">
                <h2 class="start__title">"ยินดีต้อนรับสู่ DrugItems"</h2>
                <p class="start__sub">
                    "โปรแกรมตรวจสอบว่าข้อมูลในตารางยา (drugitems) ของฐานข้อมูล MySQL ตรงกับไฟล์ snapshot หรือไม่ - อ่านอย่างเดียว ไม่มีการแก้ไขข้อมูลใดๆ"
                </p>
                <ConnectionForm state=state title="ตั้งค่าการเชื่อมต่อ MySQL" />
            </div>
        </div>
    }
}
