<div align="center">
  <h1>Gramflow</h1>
  <p><strong>A free, open source, private grammar and spell checker for the Mac and Windows apps you already write in.</strong></p>
</div>

> Gramflow is a fork of [Harper](https://github.com/automattic/harper) by Automattic, an open source project under the Apache 2.0 license. Thank you to the Harper authors and contributors for the grammar engine and the foundation this app is built on. The Humanizer feature is built on the open source [Humanizer](https://github.com/sgaofen/humanizer-local-model) by Sgaofen, with its [model](https://huggingface.co/jialinyyzz/humanizer) by jialinyyzz. See [NOTICE](NOTICE) for details.

Gramflow checks your writing as you type, in Notes, Mail, Messages, TextEdit and any other app you switch on. Underlines appear right in the text, and a click applies the fix. Everything runs on your computer: your text never goes to a server.

## Why Gramflow?

- **Free**: no subscription, no account
- **Private**: all checking happens on your own computer
- **Fast**: suggestions in milliseconds
- **Works across apps**: native apps through macOS Accessibility, websites and webmail through the browser extension

## Features

- Grammar, spelling and punctuation suggestions with one-click fixes
- A rule pack that flags AI-sounding phrases ("delve into", "rich tapestry of")
- Email tone, wordiness and common-mistake rules
- Per-app switches, so Gramflow only watches the apps you choose
- A built-in editor, a personal dictionary and per-rule settings
- Humanizer: rewrites your draft so it reads more naturally, entirely on your computer with a model you download once (optional, and skippable during setup)
- A browser extension for Chrome, Edge, Brave, Opera and Firefox, with per-profile status for Chrome, Edge and Brave in Settings → Integrations

## Download and install

Download the installer for your system (latest version, 3.0.13).

| Platform                          | Installer                                                                                                              |
| --------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| macOS Apple Silicon (M1 or newer) | [DMG](https://github.com/reachneoagent-commits/gramflow/releases/download/v3.0.13/Gramflow_3.0.13_Mac_Apple-Silicon.dmg) |
| macOS Intel                       | [DMG](https://github.com/reachneoagent-commits/gramflow/releases/download/v3.0.13/Gramflow_3.0.13_Mac_Intel.dmg)         |
| Windows x64                       | [Installer](https://github.com/reachneoagent-commits/gramflow/releases/download/v3.0.13/Gramflow_3.0.13_x64-setup.exe)   |
| Windows ARM64                     | [Installer](https://github.com/reachneoagent-commits/gramflow/releases/download/v3.0.13/Gramflow_3.0.13_arm64-setup.exe) |

Gramflow isn't signed with a paid Apple or Microsoft certificate (that costs money every year, and Gramflow is free). Your computer will warn you the first time you open it. The warning is expected, and the steps below get past it once.

### Mac

1. Download the `.dmg` that matches your Mac:
   - **`Mac_Apple-Silicon`**: Macs with an M1, M2, M3 or newer chip.
   - **`Mac_Intel`**: older Macs with an Intel chip.
2. Open the `.dmg` and drag **Gramflow** into **Applications**.
3. Open Gramflow. macOS will say it can't verify the app. Click **Done** (not Move to Trash).
4. Open **System Settings → Privacy & Security**, scroll down to the message about Gramflow, click **Open Anyway**, and enter your password.
5. When asked, allow **Accessibility** access. Gramflow needs it to read what you type and to apply fixes.

If step 4 doesn't show Open Anyway, run this once and open Gramflow again:

```bash
xattr -cr /Applications/Gramflow.app
```

If Gramflow keeps asking for Accessibility after an update, or stops underlining in Notes or Mail, run this once in Terminal:

```bash
tccutil reset Accessibility com.gramflow.app
```

Then reopen Gramflow and click **Allow**.

### Windows

1. Download the `.exe` installer (`x64-setup` for most PCs, `arm64-setup` for Windows on ARM) and run it.
2. If Windows says **"Windows protected your PC"**, click **More info**, then **Run anyway**.
3. Finish the installer and open Gramflow.

### Add the browser extension (optional)

The extension checks your writing on websites and in webmail such as Gmail. Chrome, Edge and Brave keep a separate extension list for each profile, so repeat these steps for every profile you want checked.

1. In Gramflow, open **Settings → Integrations → Browser extension**. Your profiles are listed, with a status for each.
2. Click **Show extension folder**. Gramflow copies the extension to a **Gramflow Browser Extension** folder in Documents.
3. Click **Set up** next to a profile. That profile's extensions page opens.
4. Turn on **Developer mode**, click **Load unpacked**, and choose the **Gramflow Browser Extension** folder.

The status in Settings changes to "Extension active" once the profile has it.

**Opera** uses the same extension folder: open `opera://extensions`, turn on Developer mode, click **Load unpacked**, and choose the **Gramflow Browser Extension** folder. Settings has a **Set up** button that opens that page.

**Firefox** needs its own copy of the extension:

1. In Gramflow, open **Settings → Integrations → Browser extension** and click **Set up** next to Firefox. Gramflow copies the extension to a **Gramflow Firefox Extension** folder in Documents and opens `about:debugging#/runtime/this-firefox`.
2. Click **Load Temporary Add-on…** and choose the `manifest.json` file inside that folder.

Firefox only keeps unsigned extensions until it quits, so repeat step 2 after restarting Firefox. Firefox 146 or newer is required.

## Build it yourself

You need [Rust](https://rustup.rs/), [Node.js](https://nodejs.org/), [pnpm](https://pnpm.io/), [just](https://github.com/casey/just) and [wasm-pack](https://rustwasm.github.io/wasm-pack/). Building from source avoids the unsigned-app warning, because your own computer made the app.

```bash
just build-chrome-plugin
just build-harperjs build-lint-framework build-components build-harper-editor
cd harper-desktop
pnpm install
pnpm tauri build
```

For development, run `just dev-desktop`.

## About this fork

Gramflow's grammar engine, rules and browser extension come from Harper. The original Harper README is kept in [README.harper.md](README.harper.md), and the upstream project is at <https://github.com/automattic/harper>.

### Credits

- **[Harper](https://github.com/automattic/harper)** by Automattic: the grammar engine, rules and browser extension (Apache 2.0).
- **[Humanizer](https://github.com/sgaofen/humanizer-local-model)** by Sgaofen and the **[humanizer model](https://huggingface.co/jialinyyzz/humanizer)** by jialinyyzz: the local rewriting feature (Apache 2.0). The model is downloaded by the user on their own computer and is not included in the installers.
- **[llama.cpp](https://github.com/ggml-org/llama.cpp)**: the engine that runs the Humanizer model locally (MIT).

## License

Apache License 2.0. See [LICENSE](LICENSE). Gramflow is a derivative work of Harper, Copyright (c) Automattic. "Harper" and "Automattic" names and logos are not licensed for use as Gramflow branding.
