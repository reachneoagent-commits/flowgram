# Where Gramflow works, and how we will prove it

Status key: **Built** = code exists and compiles. **Tested** = a human or script confirmed it works. Nothing below is marked Tested yet except the items under "Tested so far".

## How each surface is reached
| Surface | Mechanism | Status |
|---|---|---|
| Any native Mac app (Mail, Notes, Messages, TextEdit, Slack) | Desktop app reads text through macOS Accessibility and draws an overlay | Built (default list: TextEdit, Mail, Messages, Notes). Not tested on this build |
| Chrome, Edge, Brave, Arc | Chrome extension (`packages/chrome-plugin`) | Built upstream as "Harper". **Not rebranded**, not built or tested |
| Firefox | Same extension | Same |
| Safari | Needs a Safari Web Extension wrapped in a Mac app (Xcode, Apple Developer account) | **Not started** |
| Microsoft Word, Outlook, PowerPoint | Needs an Office add-in (Office.js). Accessibility reading of Word is unproven | **Not started** |
| Google Docs | Canvas-rendered; needs dedicated handling in the extension | **Not started** |
| Windows apps | Windows UI Automation broker | Partly built, not tested |
| Code editors | VS Code extension, `harper-ls` for Neovim, Helix, Emacs, Zed | Built upstream |

The desktop app only reads fields whose macOS role is a text area or text field. Rich editors that do not expose those roles (some web views, Word) will not be checked by the desktop app.

## Privacy rules (must hold everywhere)
- **Password fields are never read.** Fixed this session: the Mac reader now skips `AXSecureTextField` (one unit test). Before this fix, password fields were treated like any text field.
- Text stays on the device. No network calls are made to lint.
- Users can turn each app off, and pause everything from the menu bar.

## Email-specific best practices (to build or verify)
1. Do not flag quoted replies (lines starting with `>` or "On ... wrote:").
2. Ignore email addresses, URLs and signatures.
3. Check the greeting and sign-off for capitalization.
4. Catch wrong-recipient-style slips only through rules, never by reading the recipient list.
5. Keep suggestions fast (10 ms engine) so typing never lags.
6. Respect the per-app toggle: work email on, personal chat off.

## Document-writing best practices (to build or verify)
1. Readability readout (built in the editor status bar, grade level and reading time).
2. AI-sounding phrase pack (22 rules, on by default, one switch to turn off).
3. Goals presets (formal, casual, academic) that switch rule groups on or off. Not built.
4. Style guide import through Weirpacks. Built upstream, UI present.
5. Dictionary export and import. Built upstream.
6. Word add-in so the same rules run inside .docx files. Not started.

## Tested so far
- Rust: harper-core and every workspace crate pass; desktop crate 84 unit tests pass.
- Svelte type check and production build pass.
- Settings UI (light and dark), onboarding and rules pages rendered in a browser against a mocked backend and visually checked.

## Not tested yet
Real macOS permission flow, highlighting in Mail, Notes and Chrome, the menu bar and highlighter overlay, `.dmg` install, Windows, Safari, Word.

## Manual test script for the first Mac run (about 15 minutes)
1. Launch the app. Complete onboarding. Grant Accessibility.
2. TextEdit: type `This is an test.` Expect an underline on `an`. Click it, apply the fix.
3. TextEdit: type `We will delve into the data.` Expect an AI-sounding-phrase suggestion.
4. Mail: start a new message, type a misspelling. Expect an underline and a card.
5. Mail: reply to a message. Confirm quoted text is not flagged. (Expected to fail today; record it.)
6. Notes: repeat step 4.
7. Safari or Chrome: open any page with a text box (not Google Docs). Note whether the desktop app highlights it.
8. Any login page: click a password field and type. Confirm nothing is underlined or shown.
9. Settings: switch theme Light, Dark, System. Confirm every page is readable.
10. Disable Mail in Integrations. Confirm underlines stop in Mail.
11. Quit from the menu bar. Confirm highlights disappear.
Record each result as pass, fail or not applicable.
