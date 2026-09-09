//! Settings dialog - two sections:
//!
//! 1. **MySQL connection** - host/port/database/user/password (stored
//!    encrypted; reused from [`ConnectionForm`]).
//! 2. **การตั้งค่าการตรวจสอบ** - which table to compare, a site label, and
//!    the list of columns excluded from the cell-by-cell comparison.
//!
//! The default stance is "compare every column" - the snapshot is the
//! truth. Excluding columns is an explicit, optional relaxation.

use leptos::ev;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, AppSettingsInput};
use crate::components::connection_form::ConnectionForm;
use crate::components::icons::{IconCheckCircle, IconPlug, IconSave, IconX};
use crate::state::{AppState, ConnectionHealth};

/// The two settings tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsTab {
    Connection,
    App,
}

#[component]
pub fn SettingsModal(state: AppState) -> impl IntoView {
    let tab = RwSignal::new(SettingsTab::Connection);

    // --- App settings section --------------------------------------------
    let site_label = RwSignal::new(String::new());
    let table_name = RwSignal::new("drugitems".to_string());
    let ignore = RwSignal::new(Vec::<String>::new());
    let custom_col = RwSignal::new(String::new());
    let settings_message = RwSignal::new(None::<(bool, String)>);
    let settings_busy = RwSignal::new(false);

    spawn_local(async move {
        match api::get_settings().await {
            Ok(settings) => {
                site_label.set(settings.site_label.clone());
                table_name.set(settings.table_name.clone());
                ignore.set(settings.ignore_columns.clone());
                state.site_label.set(settings.site_label);
                state.table_name.set(settings.table_name);
                state.ignore_columns.set(settings.ignore_columns);
            }
            Err(e) => settings_message.set(Some((false, e.message))),
        }
    });

    let close = move || {
        state.settings_open.set(false);
    };
    let open_flag = state.settings_open;
    let close_on_escape = move |event: ev::KeyboardEvent| {
        if event.key() == "Escape" && open_flag.get_untracked() {
            close();
        }
    };
    let escape_handle = window_event_listener(ev::keydown, close_on_escape);
    let _escape_handle = StoredValue::new(escape_handle);

    let toggle_ignore = move |column: String| {
        let mut list = ignore.get_untracked();
        if let Some(pos) = list.iter().position(|c| *c == column) {
            list.remove(pos);
        } else {
            list.push(column);
        }
        list.sort();
        ignore.set(list);
    };

    let add_custom = move || {
        let value = custom_col.get_untracked().trim().to_string();
        if !value.is_empty() {
            let mut list = ignore.get_untracked();
            if !list.contains(&value) {
                list.push(value);
                list.sort();
                ignore.set(list);
            }
            custom_col.set(String::new());
        }
    };

    let run_save_settings = move || {
        if settings_busy.get_untracked() {
            return;
        }
        settings_busy.set(true);
        settings_message.set(None);
        let input = AppSettingsInput {
            table_name: table_name.get_untracked(),
            site_label: site_label.get_untracked(),
            ignore_columns: ignore.get_untracked(),
        };
        spawn_local(async move {
            match api::save_settings(&input).await {
                Ok(()) => {
                    state.site_label.set(input.site_label.clone());
                    state.table_name.set(input.table_name.clone());
                    state.ignore_columns.set(input.ignore_columns.clone());
                    settings_message.set(Some((true, "บันทึกการตั้งค่าแล้ว".to_string())));
                }
                Err(e) => settings_message.set(Some((false, e.message))),
            }
            settings_busy.set(false);
        });
    };

    let snapshot_columns = move || {
        state
            .snapshot
            .get()
            .map(|meta| meta.columns.clone())
            .unwrap_or_default()
    };

    // Reset everything - the app returns to the first-run setup screen.
    let run_clear = move || {
        spawn_local(async move {
            match api::clear_site_config().await {
                Ok(()) => {
                    state.configured.set(false);
                    state.settings_open.set(false);
                    state.report.set(None);
                    state.snapshot.set(None);
                    state.ignore_columns.set(Vec::new());
                    state.site_label.set(String::new());
                    state.table_name.set("drugitems".to_string());
                    state.health.set(ConnectionHealth::Unconfigured);
                }
                Err(e) => settings_message.set(Some((false, e.message))),
            }
        });
    };

    let app_panel = move || {
        view! {
            <section class="form-section">
                <h3 class="form-section__title">
                    <IconCheckCircle class="icon" />
                    "การตั้งค่าการตรวจสอบ"
                </h3>

                <div class="form-field">
                    <label for="cfg-label">"ชื่อสถานบริการ (แสดงบนหัวแอป)"</label>
                    <input
                        id="cfg-label"
                        class="form-input"
                        placeholder="เช่น โรงพยาบาลสมมติ"
                        prop:value=move || site_label.get()
                        on:input=move |ev| site_label.set(event_target_value(&ev))
                    />
                </div>
                <div class="form-field">
                    <label for="cfg-table">"ตารางที่เปรียบเทียบ"</label>
                    <input
                        id="cfg-table"
                        class="form-input form-input--mono"
                        placeholder="drugitems"
                        prop:value=move || table_name.get()
                        on:input=move |ev| table_name.set(event_target_value(&ev))
                    />
                </div>

                <p class="modal__note">
                    "คอลัมน์ที่ไม่ต้องการเปรียบเทียบ (ค่าเริ่มต้น: เปรียบเทียบทุกคอลัมน์ ทุกค่า ตามไฟล์ snapshot) -"
                </p>

                {move || {
                    let cols = snapshot_columns();
                    if cols.is_empty() {
                        view! {
                            <p class="modal__note">
                                "ยังไม่ได้เลือกไฟล์ snapshot - ยังไม่แสดงรายการคอลัมน์ ใช้ช่องด้านล่างเพิ่มคอลัมน์เองได้"
                            </p>
                        }.into_any()
                    } else {
                        view! {
                            <div class="ignore-grid">
                                {cols.iter().map(|column| {
                                    let col = column.clone();
                                    let display = col.clone();
                                    let checked_col = col.clone();
                                    let checked = move || {
                                        ignore.get().contains(&checked_col)
                                    };
                                    view! {
                                        <label class="ignore-item">
                                            <input
                                                type="checkbox"
                                                prop:checked=checked
                                                on:change=move |_| toggle_ignore(col.clone())
                                            />
                                            <span class="ignore-item__name">{display}</span>
                                        </label>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                }}

                <div class="form-row" style="align-items:flex-end">
                    <div class="form-field form-field--grow">
                        <label for="cfg-custom-col">"เพิ่มคอลัมน์ด้วยตนเอง"</label>
                        <input
                            id="cfg-custom-col"
                            class="form-input form-input--mono"
                            placeholder="ชื่อคอลัมน์"
                            prop:value=move || custom_col.get()
                            on:input=move |ev| custom_col.set(event_target_value(&ev))
                            on:keydown=move |ev: ev::KeyboardEvent| {
                                if ev.key() == "Enter" {
                                    add_custom();
                                }
                            }
                        />
                    </div>
                    <button
                        class="button-secondary button-secondary--inline"
                        on:click=move |_| add_custom()
                    >
                        "เพิ่ม"
                    </button>
                </div>

                {move || {
                    let list = ignore.get();
                    if list.is_empty() {
                        view! { <p class="modal__note">"ไม่มีการยกเว้นคอลัมน์ - ตรวจทุกคอลัมน์"</p> }.into_any()
                    } else {
                        view! {
                            <ul class="result-list">
                                {list.iter().map(|column| {
                                    let col = column.clone();
                                    view! {
                                        <li class="search-result-row">
                                            <span class="search-result-row__code">{col.clone()}</span>
                                            <button
                                                class="button-secondary button-secondary--inline"
                                                on:click=move |_| toggle_ignore(col.clone())
                                            >
                                                <IconX class="icon" />
                                                "ยกเลิก"
                                            </button>
                                        </li>
                                    }
                                }).collect_view()}
                            </ul>
                        }.into_any()
                    }
                }}

                {move || {
                    settings_message.get().map(|(is_success, text)| {
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
                        class="button-primary button-primary--inline"
                        on:click=move |_| run_save_settings()
                        prop:disabled=move || settings_busy.get()
                    >
                        <IconSave class="icon" />
                        "บันทึกการตั้งค่า"
                    </button>
                </div>

                <div class="modal__divider"></div>
                <button
                    class="button-danger button-danger--block"
                    on:click=move |_| run_clear()
                >
                    <IconX class="icon" />
                    "ลบการตั้งค่าทั้งหมด (กลับสู่หน้าตั้งค่าเริ่มต้น)"
                </button>
            </section>
        }
        .into_any()
    };

    view! {
        <div
            class="modal-backdrop"
            style:display=move || {
                if state.settings_open.get() { "flex" } else { "none" }
            }
            on:click=move |_| close()
        >
            <section class="modal modal--wide" on:click=move |ev| ev.stop_propagation()>
                <h2 class="modal__title">"ตั้งค่า"</h2>

                <div class="tabs">
                    <button
                        class=move || {
                            if tab.get() == SettingsTab::Connection { "tab tab--active" } else { "tab" }
                        }
                        on:click=move |_| tab.set(SettingsTab::Connection)
                    >
                        <IconPlug class="icon" />
                        "การเชื่อมต่อ"
                    </button>
                    <button
                        class=move || {
                            if tab.get() == SettingsTab::App { "tab tab--active" } else { "tab" }
                        }
                        on:click=move |_| tab.set(SettingsTab::App)
                    >
                        <IconCheckCircle class="icon" />
                        "การตั้งค่าการตรวจสอบ"
                    </button>
                </div>

                {move || {
                    if tab.get() == SettingsTab::Connection {
                        view! {
                            <>
                                <ConnectionForm state=state title="การเชื่อมต่อ MySQL" />
                                <div class="modal__actions">
                                    <button
                                        class="button-secondary button-secondary--inline"
                                        on:click=move |_| close()
                                    >
                                        <IconX class="icon" />
                                        "ปิด"
                                    </button>
                                </div>
                            </>
                        }.into_any()
                    } else {
                        view! {
                            <>
                                {app_panel()}
                                <div class="modal__actions">
                                    <button
                                        class="button-secondary button-secondary--inline"
                                        on:click=move |_| close()
                                    >
                                        <IconX class="icon" />
                                        "ปิด"
                                    </button>
                                </div>
                            </>
                        }.into_any()
                    }
                }}
            </section>
        </div>
    }
}
