# Gramflow design system (what the app actually implements)

One source of truth for tokens: `packages/components/src/lib/brand-tokens.css`. The editor imports the same file, so settings, onboarding and editor cannot drift apart. Visual reference: `brand/gramflow-brand-mockup.html`.

## Tokens (shadcn-style semantic roles, light and dark under `.dark`)
`background`, `foreground`, `card`, `muted`, `muted-foreground`, `border`, `input`, `primary` (lime), `primary-foreground` (ink), `accent-surface` (meadow), `inverse`, `ring`, `destructive`, and issue categories `cat-spelling`, `cat-grammar`, `cat-style`, `cat-clarity`. Use roles in new code (`bg-background`, `text-muted-foreground`, `border-border`, `bg-cat-spelling/15`), never raw hex.

The old Tailwind scales (stone, gray, cream, amber, emerald, primary, accent) are remapped to brand colours, so legacy classes follow the brand too.

## Rules
- Lime is a fill for ink text only. Never white text on lime, never lime text on white.
- Theme: `applyTheme()` in `harper-desktop/src/lib/theme.ts` sets `data-theme` and `.dark` on both `html` and `body`, and syncs the native title bar. Every window (settings, onboarding, editor) calls it.
- Headline pattern: sans, weight 500, tracking -5%, one serif-italic accent word (Instrument Serif, bundled locally).
- Minimum text 12 px. Targets at least 24 px; primary controls 32 to 48 px.
- Four issue categories everywhere (macOS overlay, browser underlines, editor sidebar): coral spelling, sky grammar, amber clarity, violet style.
- Booleans are switches (2 px ink border, lime when on), never bare checkboxes.
- Focus ring always visible: 3 px, ink on light, lime on dark.

## Components restyled
Button (lime primary), Toggle (switch), Checkbox, Panel, SlideDeck (onboarding: logo, serif headline accent, lime progress), Select (chevron), settings sidebar (logo, lime active row), settings page headers, editor sidebar (cards, empty state, count badge), editor status bar (readability readout).

## Not done yet
Chrome extension popup and in-page cards (use `lint-framework` colours, but their own layout), Windows-specific chrome, the menu-bar menu, the highlighter popup card in Rust.
