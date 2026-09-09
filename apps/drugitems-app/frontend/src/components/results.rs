//! Results dashboard: verdict, summary, changed-column breakdown, and the
//! filterable/expandable diff list.

use std::collections::HashSet;

use drugitems_core::{ChangeStatus, RowDiff};
use leptos::prelude::*;

use crate::components::icons::{IconAlert, IconCheckCircle, IconSearch};
use crate::state::AppState;

/// Which rows to show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Filter {
    All,
    Changed,
    Added,
    Missing,
}

impl Filter {
    fn all() -> [Filter; 4] {
        [Filter::All, Filter::Changed, Filter::Added, Filter::Missing]
    }

    fn label(self) -> &'static str {
        match self {
            Filter::All => "ทั้งหมด",
            Filter::Changed => "ต่าง",
            Filter::Added => "ใหม่ใน DB",
            Filter::Missing => "หายจาก DB",
        }
    }

    fn matches(self, status: ChangeStatus) -> bool {
        match self {
            Filter::All => true,
            Filter::Changed => status == ChangeStatus::Changed,
            Filter::Added => status == ChangeStatus::AddedInDb,
            Filter::Missing => status == ChangeStatus::MissingInDb,
        }
    }
}

/// Badge label + CSS class for a row status.
fn status_badge(status: ChangeStatus) -> (&'static str, &'static str) {
    match status {
        ChangeStatus::Unchanged => ("ตรงกัน", "badge badge--ok"),
        ChangeStatus::Changed => ("ต่าง", "badge badge--warn"),
        ChangeStatus::AddedInDb => ("ใหม่ใน DB", "badge badge--added"),
        ChangeStatus::MissingInDb => ("หายจาก DB", "badge badge--missing"),
    }
}

/// Toggle a diff row's expansion.
fn toggle_row(code: String, open: RwSignal<HashSet<String>>) {
    let mut set = open.get_untracked();
    if !set.remove(&code) {
        set.insert(code);
    }
    open.set(set);
}

#[component]
pub fn Results(state: AppState) -> impl IntoView {
    let filter = RwSignal::new(Filter::All);
    let query = RwSignal::new(String::new());
    let open = RwSignal::new(HashSet::<String>::new());

    view! {
        {move || {
            match state.report.get() {
                None => empty_view(state).into_any(),
                Some(report) => {
                    let problems = report.counts.changed
                        + report.counts.added_in_db
                        + report.counts.missing_in_db;
                    let has_problems = problems > 0;
                    let q = query.get().trim().to_lowercase();
                    let sel = filter.get();
                    let list: Vec<RowDiff> = report
                        .rows
                        .iter()
                        .filter(|r| sel.matches(r.status))
                        .filter(|r| {
                            if q.is_empty() {
                                return true;
                            }
                            let in_code = r.code.to_lowercase().contains(&q);
                            let in_name = r
                                .name
                                .as_deref()
                                .unwrap_or("")
                                .to_lowercase()
                                .contains(&q);
                            in_code || in_name
                        })
                        .cloned()
                        .collect();

                    view! {
                        <div class="results">
                            <div class=if has_problems { "verdict verdict--warn" } else { "verdict verdict--ok" }>
                                {move || {
                                    if has_problems {
                                        view! { <IconAlert class="icon" /> }.into_any()
                                    } else {
                                        view! { <IconCheckCircle class="icon" /> }.into_any()
                                    }
                                }}
                                {move || {
                                    if has_problems {
                                        format!("พบ {problems} รายการที่เปลี่ยนแปลง - ข้อมูลในฐานข้อมูลไม่ตรงกับไฟล์")
                                    } else {
                                        "ไม่พบการเปลี่ยนแปลง - ข้อมูลในฐานข้อมูลตรงกับไฟล์ทุกคอลัมน์ ทุกค่า".to_string()
                                    }
                                }}
                            </div>

                            <div class="summary">
                                <div class="summary__item">
                                    <span class="summary__num">{report.snapshot_rows}</span>
                                    <span class="summary__label">"ในไฟล์"</span>
                                </div>
                                <div class="summary__item">
                                    <span class="summary__num">{report.db_rows}</span>
                                    <span class="summary__label">"ในฐานข้อมูล"</span>
                                </div>
                                <div class="summary__item">
                                    <span class="summary__num summary__num--ok">{report.counts.unchanged}</span>
                                    <span class="summary__label">"ตรงกัน"</span>
                                </div>
                                <div class="summary__item">
                                    <span class="summary__num summary__num--warn">{report.counts.changed}</span>
                                    <span class="summary__label">"ต่าง"</span>
                                </div>
                                <div class="summary__item">
                                    <span class="summary__num summary__num--added">{report.counts.added_in_db}</span>
                                    <span class="summary__label">"ใหม่ใน DB"</span>
                                </div>
                                <div class="summary__item">
                                    <span class="summary__num summary__num--missing">{report.counts.missing_in_db}</span>
                                    <span class="summary__label">"หายจาก DB"</span>
                                </div>
                            </div>

                            {move || {
                                if !report.changed_columns.is_empty() {
                                    view! {
                                        <div class="chips">
                                            {report.changed_columns.iter().map(|c| {
                                                let col = c.column.clone();
                                                let n = c.changed_rows;
                                                view! { <span class="chip">{format!("{col} · {n}")}</span> }
                                            }).collect_view()}
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <span hidden></span> }.into_any()
                                }
                            }}

                            {move || {
                                if !report.schema_notes.is_empty() {
                                    view! {
                                        <div class="schema-notes">
                                            {report.schema_notes.iter().map(|n| {
                                                let note = n.clone();
                                                view! { <p class="schema-note">{note}</p> }
                                            }).collect_view()}
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <span hidden></span> }.into_any()
                                }
                            }}

                            <div class="toolbar">
                                <div class="segmented">
                                    {Filter::all().iter().map(|f| {
                                        let f = *f;
                                        view! {
                                            <button
                                                class=move || {
                                                    if filter.get() == f {
                                                        "segmented__btn segmented__btn--active"
                                                    } else {
                                                        "segmented__btn"
                                                    }
                                                }
                                                on:click=move |_| filter.set(f)
                                            >
                                                {f.label()}
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
                                <div class="search-wrapper">
                                    <IconSearch class="search-icon" />
                                    <input
                                        class="search-input"
                                        placeholder="ค้นหา icode หรือชื่อยา…"
                                        prop:value=move || query.get()
                                        on:input=move |ev| query.set(event_target_value(&ev))
                                    />
                                </div>
                            </div>

                            <p class="toolbar__count">
                                {format!("แสดง {} รายการ", list.len())}
                            </p>

                            <div class="diff-list">
                                {list.iter().map(|row| {
                                    let code = row.code.clone();
                                    let click_code = code.clone();
                                    let text_code = code.clone();
                                    let chev_code = code.clone();
                                    let body_code = code.clone();
                                    let name = row.name.clone().unwrap_or_default();
                                    let change_count = row.changes.len();
                                    let (label, badge_cls) = status_badge(row.status);
                                    let row_owned = row.clone();
                                    view! {
                                        <div class="diff-row">
                                            <button class="diff-row__head" on:click=move |_| toggle_row(click_code.clone(), open)>
                                                <span class={badge_cls}>{label}</span>
                                                <span class="diff-row__code">{text_code}</span>
                                                <span class="diff-row__name">{name}</span>
                                                {move || {
                                                    if change_count > 0 {
                                                        view! {
                                                            <span class="diff-row__count">{format!("{} คอลัมน์", change_count)}</span>
                                                        }.into_any()
                                                    } else {
                                                        view! { <span hidden></span> }.into_any()
                                                    }
                                                }}
                                                <span class="diff-row__chevron">
                                                    {move || if open.get().contains(&chev_code) { "▾" } else { "▸" }}
                                                </span>
                                            </button>

                                            {move || {
                                                if open.get().contains(&body_code) {
                                                    view! {
                                                        <div class="diff-row__body">
                                                            <table class="diff-table">
                                                                <thead>
                                                                    <tr>
                                                                        <th>"คอลัมน์"</th>
                                                                        <th>"ค่าตามไฟล์ (source of truth)"</th>
                                                                        <th>"ค่าในฐานข้อมูล (ปัจจุบัน)"</th>
                                                                    </tr>
                                                                </thead>
                                                                <tbody>
                                                                    {row_owned.changes.iter().map(|c| {
                                                                        let col = c.column.clone();
                                                                        let exp = c.expected.text.clone();
                                                                        let act = c.actual.text.clone();
                                                                        let exp_empty = c.expected.empty;
                                                                        let act_empty = c.actual.empty;
                                                                        view! {
                                                                            <tr>
                                                                                <td class="diff-table__col">{col}</td>
                                                                                <td class="diff-table__cell">
                                                                                    {move || if exp_empty { "-".to_string() } else { exp.clone() }}
                                                                                </td>
                                                                                <td class="diff-table__cell diff-table__cell--actual">
                                                                                    {move || if act_empty { "-".to_string() } else { act.clone() }}
                                                                                </td>
                                                                            </tr>
                                                                        }
                                                                    }).collect_view()}
                                                                </tbody>
                                                            </table>
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! { <span hidden></span> }.into_any()
                                                }
                                            }}
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                    }.into_any()
                }
            }
        }}
    }
}

/// Shown before the first comparison runs.
fn empty_view(state: AppState) -> impl IntoView {
    view! {
        <div class="results results--empty">
            <p class="results__empty-title">"ยังไม่มีผลการเปรียบเทียบ"</p>
            <p class="panel__hint">
                {move || {
                    if state.snapshot.get().is_some() {
                        "เลือกไฟล์ snapshot แล้ว - กดปุ่ม เปรียบเทียบข้อมูล"
                    } else {
                        "เลือกไฟล์ snapshot (source of truth) ก่อน แล้วกด เปรียบเทียบข้อมูล"
                    }
                }}
            </p>
            {move || {
                if let Some(err) = state.compare_error.get() {
                    view! { <p class="form-message form-message--error">{err}</p> }.into_any()
                } else {
                    view! { <span hidden></span> }.into_any()
                }
            }}
        </div>
    }
}
