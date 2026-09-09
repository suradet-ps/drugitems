//! Lucide-style SVG icon components.
//!
//! Stroke-based icons (24×24 viewBox, `currentColor`, 2px stroke, round
//! caps) - render crisply at any size and inherit their color from the
//! surrounding text. Icons are decorative: `aria-hidden` and never carry
//! text alternatives; nearby text always explains the state.

use leptos::prelude::*;

/// Generates one icon component: an `<svg>` shell with the given nodes.
macro_rules! icon {
    ($(#[$doc:meta])* $name:ident, $($nodes:tt)*) => {
        $(#[$doc])*
        #[component]
        pub fn $name(class: &'static str) -> impl IntoView {
            view! {
                <svg
                    class=class
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    aria-hidden="true"
                >
                    $($nodes)*
                </svg>
            }
        }
    };
}

icon!(
    /// Pill/capsule - the DrugItems brand mark.
    IconLogo,
    <path d="M10.5 20.5a5.5 5.5 0 0 1-7.7-7.7l8.4-8.4a5.5 5.5 0 0 1 7.7 7.7l-8.4 8.4Z" />
    <path d="M8.3 8.3l7.4 7.4" />
    <circle cx="5.8" cy="15.7" r="1.2" />
    <circle cx="15.7" cy="5.8" r="1.2" />
);

icon!(
    /// Magnifying glass - search actions.
    IconSearch,
    <circle cx="11" cy="11" r="8" />
    <path d="m21 21-4.3-4.3" />
);

icon!(
    /// Gear - settings.
    IconSettings,
    <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
    <circle cx="12" cy="12" r="3" />
);

icon!(
    /// Plug - test connection.
    IconPlug,
    <path d="M12 22v-5" />
    <path d="M9 8V2" />
    <path d="M15 8V2" />
    <path d="M18 8v5a4 4 0 0 1-4 4h-4a4 4 0 0 1-4-4V8Z" />
);

icon!(
    /// Save (floppy disk).
    IconSave,
    <path d="M15.2 3a2 2 0 0 1 1.4.6l3.8 3.8a2 2 0 0 1 .6 1.4V19a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z" />
    <path d="M17 21v-7a1 1 0 0 0-1-1H8a1 1 0 0 0-1 1v7" />
    <path d="M7 3v4a1 1 0 0 0 1 1h7" />
);

icon!(
    /// X - close.
    IconX,
    <path d="M18 6 6 18" />
    <path d="m6 6 12 12" />
);

icon!(
    /// Question mark in a circle - help.
    IconHelp,
    <circle cx="12" cy="12" r="10" />
    <path d="M9.1 9a3 3 0 0 1 5.8 1c0 2-3 3-3 3" />
    <path d="M12 17h.01" />
);

icon!(
    /// Document - snapshot file.
    IconFile,
    <path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z" />
    <path d="M14 2v4a2 2 0 0 0 2 2h4" />
    <path d="M8 13h8" />
    <path d="M8 17h5" />
);

icon!(
    /// Refresh/compare arrows.
    IconRefresh,
    <path d="M3 12a9 9 0 0 1 15-6.7L21 8" />
    <path d="M21 3v5h-5" />
    <path d="M21 12a9 9 0 0 1-15 6.7L3 16" />
    <path d="M3 21v-5h5" />
);

icon!(
    /// Download/save report.
    IconDownload,
    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
    <path d="m7 10 5 5 5-5" />
    <path d="M12 15V3" />
);

icon!(
    /// Alert triangle - problems found.
    IconAlert,
    <path d="m21.7 18-8-14a2 2 0 0 0-3.4 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.7-3Z" />
    <path d="M12 9v4" />
    <path d="M12 17h.01" />
);

icon!(
    /// Check circle - all good.
    IconCheckCircle,
    <circle cx="12" cy="12" r="10" />
    <path d="m9 12 2 2 4-4" />
);

icon!(
    /// Database - MySQL target.
    IconDatabase,
    <ellipse cx="12" cy="5" rx="9" ry="3" />
    <path d="M3 5v14a9 3 0 0 0 18 0V5" />
    <path d="M3 12a9 3 0 0 0 18 0" />
);
