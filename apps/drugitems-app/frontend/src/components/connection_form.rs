//! MySQL connection form - reused by the first-run setup screen and the
//! settings dialog. Test runs against the typed values before anything is
//! saved; credentials are encrypted at rest by the backend.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, ConnectionInput};
use crate::components::icons::{IconPlug, IconSave};
use crate::state::{AppState, ConnectionHealth};

/// Default MySQL port, prefilled in the port field.
const DEFAULT_PORT: u16 = 3306;

/// Hard ceiling for any backend operation so the busy state can never stick
/// forever if the backend future is dropped.
const OPERATION_TIMEOUT_SECS: u64 = 25;

/// Arm a hard timeout for a backend operation.
///
/// Returns a generation token; only the caller of the current token may
/// update the busy/message signals, so a stale (late-arriving) result or a
/// dropped future can never leave the UI stuck.
fn arm_operation_timeout(
    generation: RwSignal<u64>,
    busy: RwSignal<bool>,
    message: RwSignal<Option<(bool, String)>>,
) -> u64 {
    let token = generation.get_untracked() + 1;
    generation.set(token);
    set_timeout(
        move || {
            if generation.get_untracked() == token {
                busy.set(false);
                message.set(Some((
                    false,
                    "การดำเนินการใช้เวลานานเกินไป (เกิน 25 วินาที) - ตรวจสอบ Host/Port/เครือข่าย แล้วลองใหม่"
                        .to_string(),
                )));
            }
        },
        std::time::Duration::from_secs(OPERATION_TIMEOUT_SECS),
    );
    token
}

/// Zeroize an operator-typed field before dropping it.
fn wipe_field(signal: &RwSignal<String>) {
    let mut value = signal.get_untracked();
    if !value.is_empty() {
        unsafe { value.as_mut_vec().fill(0) };
    }
    signal.set(String::new());
}

/// A connection form. `title` is shown above the fields (e.g. on the
/// first-run setup screen).
#[component]
pub fn ConnectionForm(state: AppState, title: &'static str) -> impl IntoView {
    let host = RwSignal::new(String::new());
    let port = RwSignal::new(DEFAULT_PORT.to_string());
    let database = RwSignal::new("hos".to_string());
    let user = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let message = RwSignal::new(None::<(bool, String)>);
    let busy = RwSignal::new(false);
    let generation = RwSignal::new(0u64);
    on_cleanup(move || wipe_field(&password));

    // Prefill from the saved (non-secret) connection on mount.
    spawn_local(async move {
        if let Ok(info) = api::get_connection().await {
            host.set(info.host);
            port.set(info.port.to_string());
            database.set(info.database);
            user.set(info.user);
        }
    });

    let build_input = move || -> Result<ConnectionInput, String> {
        let port_value = port
            .get_untracked()
            .trim()
            .parse::<u16>()
            .map_err(|_| "พอร์ตไม่ถูกต้อง".to_string())?;
        let input = ConnectionInput {
            host: host.get_untracked(),
            port: port_value,
            database: database.get_untracked(),
            user: user.get_untracked(),
            password: password.get_untracked(),
        };
        if input.host.trim().is_empty()
            || input.database.trim().is_empty()
            || input.user.trim().is_empty()
        {
            return Err("กรอก Host, Database, User ให้ครบ".to_string());
        }
        Ok(input)
    };

    let run_test = move || {
        if busy.get_untracked() {
            return;
        }
        let input = match build_input() {
            Ok(input) => input,
            Err(err_message) => {
                message.set(Some((false, err_message)));
                return;
            }
        };
        busy.set(true);
        message.set(None);
        let token = arm_operation_timeout(generation, busy, message);
        spawn_local(async move {
            let result = api::test_connection(Some(&input)).await;
            if generation.get_untracked() == token {
                match result {
                    Ok(test) => message.set(Some((
                        true,
                        format!("เชื่อมต่อได้ (latency {} ms)", test.latency_ms),
                    ))),
                    Err(error) => message.set(Some((false, error.message))),
                }
                busy.set(false);
            }
        });
    };

    let run_save = move || {
        if busy.get_untracked() {
            return;
        }
        let input = match build_input() {
            Ok(input) => input,
            Err(err_message) => {
                message.set(Some((false, err_message)));
                return;
            }
        };
        busy.set(true);
        message.set(None);
        let token = arm_operation_timeout(generation, busy, message);
        spawn_local(async move {
            let result = api::save_connection(&input).await;
            if generation.get_untracked() == token {
                match result {
                    Ok(()) => {
                        message.set(Some((true, "บันทึกการตั้งค่าและเชื่อมต่อแล้ว".to_string())));
                        state.configured.set(true);
                        state.health.set(ConnectionHealth::Connected);
                    }
                    Err(error) => message.set(Some((false, error.message))),
                }
                busy.set(false);
            }
        });
    };

    view! {
        <section class="form-section">
            <h3 class="form-section__title">
                <IconPlug class="icon" />
                {title}
            </h3>

            <div class="form-field">
                <label for="cfg-host">"Host"</label>
                <input
                    id="cfg-host"
                    class="form-input form-input--mono"
                    placeholder="192.168.1.10"
                    prop:value=move || host.get()
                    on:input=move |ev| host.set(event_target_value(&ev))
                />
            </div>
            <div class="form-row">
                <div class="form-field" style="max-width:110px">
                    <label for="cfg-port">"Port"</label>
                    <input
                        id="cfg-port"
                        class="form-input form-input--mono"
                        prop:value=move || port.get()
                        on:input=move |ev| port.set(event_target_value(&ev))
                    />
                </div>
                <div class="form-field form-field--grow">
                    <label for="cfg-database">"Database"</label>
                    <input
                        id="cfg-database"
                        class="form-input form-input--mono"
                        placeholder="hos"
                        prop:value=move || database.get()
                        on:input=move |ev| database.set(event_target_value(&ev))
                    />
                </div>
            </div>
            <div class="form-field">
                <label for="cfg-user">"User"</label>
                <input
                    id="cfg-user"
                    class="form-input form-input--mono"
                    placeholder="drug_ro (แนะนำบัญชีอ่านอย่างเดียว)"
                    prop:value=move || user.get()
                    on:input=move |ev| user.set(event_target_value(&ev))
                />
            </div>
            <div class="form-field">
                <label for="cfg-password">"Password"</label>
                <input
                    id="cfg-password"
                    class="form-input form-input--mono"
                    type="password"
                    prop:value=move || password.get()
                    on:input=move |ev| password.set(event_target_value(&ev))
                />
            </div>

            {move || {
                message.get().map(|(is_success, text)| {
                    let class = if is_success {
                        "form-message form-message--success"
                    } else {
                        "form-message form-message--error"
                    };
                    view! { <p class=class>{text}</p> }
                })
            }}

            <div class="modal__actions" style="margin-top:var(--sp-md)">
                <button
                    class="button-secondary button-secondary--inline"
                    on:click=move |_| run_test()
                    prop:disabled=move || busy.get()
                >
                    <IconPlug class="icon" />
                    {move || if busy.get() { "กำลังทดสอบ…" } else { "ทดสอบ" }}
                </button>
                <button
                    class="button-primary button-primary--inline"
                    on:click=move |_| run_save()
                    prop:disabled=move || busy.get()
                >
                    <IconSave class="icon" />
                    {move || if busy.get() { "กำลังบันทึก…" } else { "บันทึกและเชื่อมต่อ" }}
                </button>
            </div>
        </section>
    }
}
