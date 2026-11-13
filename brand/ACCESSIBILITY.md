# Gramflow UI: accessibility and UX checklist (WCAG 2.2 AA target)

Verified in `gramflow-brand-mockup.html` (contrast measured, not estimated).

## Color
| Token | Value | Use | Ratio |
|---|---|---|---|
| ink | `#2B2C23` | text | 14.1 on white, 9.7 on lime, 10.9 on meadow |
| ink-2 | `#5B5E50` | secondary text | 6.6 white, 5.1 meadow, 4.7 fern |
| ink-3 | `#6B6E60` | hints (was `#8D9082`, 3.3, failed) | 5.2 white, 4.9 cream |
| coral / sky / violet / amber | `#C8401F` `#1F6FD1` `#6D3FD9` `#9A6200` | category text and underlines | 5.0 / 4.9 / 6.2 / 5.1 on white |

- Lime `#B8E62E` is 1.4:1 on white. It is a fill for dark-ink text only. Never use it for text, borders or icons on light surfaces.
- Never rely on color alone (1.4.1): categories also differ by underline style (wavy, solid, dotted, dashed) and always have a text label in the popup.
- Toggles carry a 2px ink border (1.4.11 non-text contrast) because lime and cream alone are 1.3:1.

## Keyboard and focus
- Visible 3px ink focus ring on every control, lime on dark surfaces (2.4.7, 2.4.11).
- Settings nav is a tablist (arrows, Home, End). Toggles are `role=switch` with name and `aria-checked`. Space and Enter activate.
- Escape dismisses the suggestion popup. The overlay must never steal focus from the user's app.
- Skip link, landmarks (`main`, `header`, `footer`), one h1, no skipped heading levels.

## Targets and type
- Targets at least 24x24 CSS px (2.5.8). Primary controls are 40-48.
- Smallest text is 12px; body 14-17px. Must survive 200% zoom and text-spacing overrides (1.4.4, 1.4.12) without clipping.
- Display type uses tight tracking (-5 to -6.5%) only at 36px and up. Serif italic is for one headline phrase, never UI or body.

## Motion and modes
- `prefers-reduced-motion` disables transitions; `forced-colors` keeps borders on controls.
- No auto-moving content. Suggestions appear on demand or after the debounce delay.

## To build in the real app (not provable in HTML)
1. macOS: set `accessibilityLabel`, role and value on every Tauri/egui overlay element. egui overlays are invisible to VoiceOver by default, so expose lint text via a Tauri window or an announced live region.
2. Announce "N suggestions" politely (`aria-live=polite`) rather than per keystroke.
3. Overlay popup: hover and focus content must be dismissible, hoverable and persistent (1.4.13). Respect macOS Reduce Motion, Increase Contrast and Reduce Transparency; the frosted feed card needs an opaque fallback.
4. Dark theme: re-derive every ratio. Do not just invert.
5. Test with VoiceOver, Full Keyboard Access, 200% text size and Windows High Contrast.
6. Privacy and error copy: plain language, say what happened and how to fix it (3.3.1, 3.3.3).
