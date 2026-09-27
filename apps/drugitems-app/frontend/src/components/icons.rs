//! SVG icon components.
//!
//! Lucide-style stroke icons (24×24 viewBox, `currentColor`, 2px stroke,
//! round caps) - render crisply at any size and inherit their color from
//! the surrounding text. `IconLogo` is the exception: the full-color brand
//! mark mirroring `icon-master.svg`. Icons are decorative: `aria-hidden`
//! and never carry text alternatives; nearby text always explains the
//! state.

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

/// The DrugItems brand mark - the full-color app icon (`icon-master.svg`:
/// neumorphic card, 6×6 status matrix, slate check badge), embedded inline
/// so it scales with CSS.
#[component]
pub fn IconLogo(class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 512 512" aria-hidden="true">
            <defs>
                <linearGradient id="neuBg" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stop-color="#F0F4F8" />
                    <stop offset="100%" stop-color="#E2E8F0" />
                </linearGradient>
                <linearGradient id="emeraldActive" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stop-color="#34D399" />
                    <stop offset="100%" stop-color="#059669" />
                </linearGradient>
                <linearGradient id="redAlert" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stop-color="#F87171" />
                    <stop offset="100%" stop-color="#DC2626" />
                </linearGradient>
                <linearGradient id="slateBadge" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stop-color="#94A3B8" />
                    <stop offset="100%" stop-color="#475569" />
                </linearGradient>
                <filter id="neuShadowApp" x="-20%" y="-20%" width="140%" height="140%">
                    <feDropShadow dx="12" dy="16" stdDeviation="18" flood-color="#94A3B8" flood-opacity="0.5" />
                    <feDropShadow dx="-10" dy="-10" stdDeviation="16" flood-color="#FFFFFF" flood-opacity="0.9" />
                </filter>
                <filter id="neuBadgeShadow" x="-20%" y="-20%" width="140%" height="140%">
                    <feDropShadow dx="8" dy="12" stdDeviation="12" flood-color="#475569" flood-opacity="0.35" />
                    <feDropShadow dx="-4" dy="-4" stdDeviation="8" flood-color="#FFFFFF" flood-opacity="0.8" />
                </filter>
                <filter id="dotShadow" x="-30%" y="-30%" width="160%" height="160%">
                    <feDropShadow dx="2" dy="4" stdDeviation="4" flood-color="#0F172A" flood-opacity="0.2" />
                </filter>
            </defs>

            <rect x="16" y="16" width="480" height="480" rx="96" fill="url(#neuBg)" filter="url(#neuShadowApp)" />
            <rect x="24" y="24" width="464" height="464" rx="88" fill="none" stroke="#FFFFFF" stroke-width="3" opacity="0.8" />

            <g transform="translate(86, 86)">
                <circle cx="0" cy="0" r="18" fill="#CBD5E1" opacity="0.7" />
                <circle cx="68" cy="0" r="18" fill="#CBD5E1" opacity="0.7" />
                <circle cx="136" cy="0" r="18" fill="#CBD5E1" opacity="0.7" />
                <circle cx="204" cy="0" r="18" fill="#CBD5E1" opacity="0.7" />
                <circle cx="272" cy="0" r="18" fill="#CBD5E1" opacity="0.7" />
                <circle cx="340" cy="0" r="18" fill="#CBD5E1" opacity="0.7" />

                <circle cx="0" cy="68" r="18" fill="#CBD5E1" opacity="0.7" />
                <circle cx="68" cy="68" r="22" fill="url(#emeraldActive)" filter="url(#dotShadow)" />
                <circle cx="136" cy="68" r="22" fill="url(#emeraldActive)" filter="url(#dotShadow)" />
                <circle cx="204" cy="68" r="22" fill="url(#emeraldActive)" filter="url(#dotShadow)" />
                <circle cx="272" cy="68" r="18" fill="#CBD5E1" opacity="0.7" />
                <circle cx="340" cy="68" r="18" fill="#CBD5E1" opacity="0.7" />

                <circle cx="0" cy="136" r="18" fill="#CBD5E1" opacity="0.7" />
                <circle cx="68" cy="136" r="22" fill="url(#emeraldActive)" filter="url(#dotShadow)" />
                <circle cx="136" cy="136" r="26" fill="url(#redAlert)" filter="url(#dotShadow)" />
                <circle cx="204" cy="136" r="22" fill="url(#emeraldActive)" filter="url(#dotShadow)" />
                <circle cx="272" cy="136" r="18" fill="#CBD5E1" opacity="0.7" />
                <circle cx="340" cy="136" r="18" fill="#CBD5E1" opacity="0.7" />

                <circle cx="0" cy="204" r="18" fill="#CBD5E1" opacity="0.7" />
                <circle cx="68" cy="204" r="22" fill="url(#emeraldActive)" filter="url(#dotShadow)" />
                <circle cx="136" cy="204" r="22" fill="url(#emeraldActive)" filter="url(#dotShadow)" />
                <circle cx="204" cy="204" r="22" fill="url(#emeraldActive)" filter="url(#dotShadow)" />
                <circle cx="272" cy="204" r="18" fill="#CBD5E1" opacity="0.7" />
                <circle cx="340" cy="204" r="18" fill="#CBD5E1" opacity="0.7" />

                <circle cx="0" cy="272" r="18" fill="#CBD5E1" opacity="0.7" />
                <circle cx="68" cy="272" r="18" fill="#CBD5E1" opacity="0.7" />
                <circle cx="136" cy="272" r="18" fill="#CBD5E1" opacity="0.7" />
                <circle cx="204" cy="272" r="18" fill="#CBD5E1" opacity="0.7" />
                <circle cx="272" cy="272" r="18" fill="#CBD5E1" opacity="0.7" />
                <circle cx="340" cy="272" r="18" fill="#CBD5E1" opacity="0.7" />

                <circle cx="0" cy="340" r="18" fill="#CBD5E1" opacity="0.7" />
                <circle cx="68" cy="340" r="18" fill="#CBD5E1" opacity="0.7" />
                <circle cx="136" cy="340" r="18" fill="#CBD5E1" opacity="0.7" />
                <circle cx="204" cy="340" r="18" fill="#CBD5E1" opacity="0.7" />
                <circle cx="272" cy="340" r="18" fill="#CBD5E1" opacity="0.7" />
                <circle cx="340" cy="340" r="18" fill="#CBD5E1" opacity="0.7" />
            </g>

            <g filter="url(#neuBadgeShadow)">
                <circle cx="396" cy="396" r="78" fill="url(#slateBadge)" stroke="#FFFFFF" stroke-width="10" />
                <path d="M 356 396 L 382 422 L 436 368" fill="none" stroke="#FFFFFF" stroke-width="16" stroke-linecap="round" stroke-linejoin="round" />
            </g>
        </svg>
    }
}

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
