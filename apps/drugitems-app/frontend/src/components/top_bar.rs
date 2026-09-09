//! Top bar - brand, live MySQL status dot, settings/help buttons.

use leptos::prelude::*;

use crate::components::icons::{IconHelp, IconLogo, IconSettings};
use crate::state::{AppState, ConnectionHealth};

#[component]
pub fn TopBar(state: AppState) -> impl IntoView {
    let on_settings = move |_| state.settings_open.set(true);
    let on_help = move |_| state.help_open.set(true);

    let brand = move || {
        let label = state.site_label.get();
        if label.is_empty() {
            "DrugItems".to_string()
        } else {
            format!("DrugItems · {label}")
        }
    };

    let status_text = move || match state.health.get() {
        ConnectionHealth::Connected => "เชื่อมต่อแล้ว".to_string(),
        ConnectionHealth::Disconnected => "ไม่สามารถเชื่อมต่อได้".to_string(),
        ConnectionHealth::Unconfigured => "ยังไม่ได้ตั้งค่า".to_string(),
    };

    view! {
        <header class="top-bar">
            <div class="top-bar__left">
                <IconLogo class="top-bar__logo" />
                <h1 class="top-bar__title">{brand}</h1>
            </div>
            <div class="top-bar__right">
                <span class="top-bar__status">
                    <span
                        class:top-bar__status-dot=true
                        class:top-bar__status-dot--disconnected=move || {
                            state.health.get() == ConnectionHealth::Disconnected
                        }
                        class:top-bar__status-dot--unconfigured=move || {
                            state.health.get() == ConnectionHealth::Unconfigured
                        }
                    ></span>
                    <span class="top-bar__status-text">{status_text}</span>
                </span>
                <button class="top-bar__button" on:click=on_help>
                    <IconHelp class="icon" />
                    "วิธีใช้"
                </button>
                <button class="top-bar__button" on:click=on_settings>
                    <IconSettings class="icon" />
                    "ตั้งค่า"
                </button>
            </div>
        </header>
    }
}
