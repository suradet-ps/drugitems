//! Left panel: snapshot selection + the "compare now" controls.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api;
use crate::components::icons::{IconDatabase, IconDownload, IconFile, IconRefresh};
use crate::state::AppState;

#[component]
pub fn SnapshotPanel(state: AppState) -> impl IntoView {
    let choose = move || {
        spawn_local(async move {
            match api::choose_snapshot().await {
                Ok(Some(meta)) => {
                    state.snapshot.set(Some(meta));
                    state.report.set(None);
                    state.compare_error.set(None);
                    state
                        .last_action
                        .set(Some("เลือกไฟล์ snapshot แล้ว - กด เปรียบเทียบข้อมูล เพื่อตรวจสอบ".to_string()));
                }
                Ok(None) => {}
                Err(e) => state.compare_error.set(Some(e.message)),
            }
        });
    };

    let run = move || {
        if state.comparing.get_untracked() {
            return;
        }
        state.comparing.set(true);
        state.compare_error.set(None);
        spawn_local(async move {
            match api::run_compare().await {
                Ok(report) => {
                    state.report.set(Some(report));
                    state.last_action.set(Some("เปรียบเทียบเสร็จสิ้น".to_string()));
                }
                Err(e) => state.compare_error.set(Some(e.message)),
            }
            state.comparing.set(false);
        });
    };

    let export = move || {
        spawn_local(async move {
            match api::export_report().await {
                Ok(path) => {
                    state
                        .last_action
                        .set(Some(format!("บันทึกรายงานแล้ว: {path}")))
                }
                Err(e) => state.compare_error.set(Some(e.message)),
            }
        });
    };

    view! {
        <div class="panel">
            <h2 class="panel__title">
                <IconFile class="icon" />
                "1 · ไฟล์ snapshot"
                <span class="pill-tag pill-tag--coral">"SOURCE OF TRUTH"</span>
            </h2>
            <p class="panel__hint">
                "ไฟล์ที่ส่งออกจากตารางยาในอดีต - ทุกคอลัมน์ ทุกค่าในไฟล์นี้คือข้อมูลที่เชื่อถือได้"
            </p>

            {move || {
                match state.snapshot.get() {
                    None => view! {
                        <div class="empty-slot">
                            <button class="button-primary button-primary--block" on:click=move |_| choose()>
                                <IconFile class="icon" />
                                "เลือกไฟล์ Excel (.xls / .xlsx)"
                            </button>
                        </div>
                    }.into_any(),
                    Some(meta) => {
                        let file = meta.file_name.clone().unwrap_or_default();
                        let rows = meta.rows;
                        let cols = meta.columns.len();
                        view! {
                            <div class="snapshot-card">
                                <div class="snapshot-card__file">
                                    <IconFile class="icon" />
                                    <span class="snapshot-card__name">{file}</span>
                                </div>
                                <div class="snapshot-card__meta">
                                    <span>{format!("{rows} รายการ")}</span>
                                    <span>{format!("{cols} คอลัมน์")}</span>
                                </div>
                                <button class="button-secondary button-secondary--block" on:click=move |_| choose()>
                                    <IconFile class="icon" />
                                    "เลือกไฟล์ใหม่"
                                </button>
                            </div>
                        }.into_any()
                    }
                }
            }}

            <div class="panel__divider"></div>

            <h2 class="panel__title">
                <IconDatabase class="icon" />
                "2 · เปรียบเทียบกับฐานข้อมูล"
            </h2>
            <p class="panel__hint">
                {move || format!("อ่านตาราง {} จาก MySQL (อ่านอย่างเดียว) แล้วเปรียบเทียบทีละคอลัมน์", state.table_name.get())}
            </p>

            {move || {
                if let Some(err) = state.compare_error.get() {
                    view! { <p class="form-message form-message--error">{err}</p> }.into_any()
                } else if let Some(action) = state.last_action.get() {
                    view! { <p class="form-message form-message--success">{action}</p> }.into_any()
                } else {
                    view! { <span hidden></span> }.into_any()
                }
            }}

            <div class="panel__actions">
                <button
                    class="button-primary button-primary--block"
                    on:click=move |_| run()
                    prop:disabled=move || state.comparing.get() || state.snapshot.get().is_none()
                >
                    <IconRefresh class="icon" />
                    {move || if state.comparing.get() { "กำลังเปรียบเทียบ…" } else { "เปรียบเทียบข้อมูล" } }
                </button>
                <button
                    class="button-secondary button-secondary--block"
                    on:click=move |_| export()
                    prop:disabled=move || state.report.get().is_none()
                >
                    <IconDownload class="icon" />
                    "บันทึกรายงาน CSV"
                </button>
            </div>
        </div>
    }
}
