# Goose - Local Companion for WHOOP 5.0

> ## 🍴 This is a fork of [b-nnett/goose](https://github.com/b-nnett/goose)
> **All original design, architecture, and source code are the work of [@b-nnett](https://github.com/b-nnett)**, the creator of Goose — full credit belongs to the upstream project. This fork does **not** claim authorship of Goose; it adds a hardening pass and documents it. See **[Improvements in this fork](#-improvements-in-this-fork)** below, or the full **[Goose Rescue write-up](docs/goose-rescue/README.md)** ([PDF](docs/goose-rescue/goose-rescue.pdf)).

**Alpha proof of concept. This build is for developers to evaluate whether a project of this scope is viable. It is not ready to use as an app for tracking personal health data yet.**

If you don't know what Xcode is, or how to build the Rust core, this build is not for you. Come back on 13 June 2026 for the first public beta on TestFlight.

![Goose app hero showing a connected WHOOP 5.0 device](docs/assets/readme-hero.png)

This prototype targets WHOOP 5.0 only. Other WHOOP generations are not supported in this build.

The app and backend have had very little attention put into performance. The app will lag, very considerably. Performance PRs are welcome, or you can wait until I address it in due course.

Goose is a local-first WHOOP 5.0 data and health metrics project. The iOS app connects to WHOOP 5.0 bands, routes packet data through the Goose Rust core, and turns that data into daily health, recovery, sleep, strain, stress, cardio, energy, coach, and debug views.

## 🪿 Improvements in this fork

*Credit first: the entire app below is [@b-nnett](https://github.com/b-nnett)'s work.* This fork added a human-directed hardening + compliance pass on top of it. Full editorial write-up (with code): **[docs/goose-rescue](docs/goose-rescue/README.md)** · **[PDF](docs/goose-rescue/goose-rescue.pdf)**.

**What was fixed — and why each one mattered**

Every item below was verified by re-running the exact command on the committed `main`, not from memory.

1. **FFI panic-safety — `panic="abort"` → `panic="unwind"` + `catch_unwind` on both `extern "C"` entry points** (`Rust/core/src/bridge.rs`, `Cargo.toml`).
   *Why it mattered:* the Rust core does all the math behind a C/FFI boundary the Swift app calls on every BLE packet. With `panic="abort"`, **any** unexpected input that panicked the core (a malformed WHOOP packet, an arithmetic edge case) took the **entire app down with a `SIGABRT`** — a black-screen crash with nothing the user or a log could show. Now a core panic is caught and returned as a structured `bridge_error` JSON the Swift side can surface. The single most fragile seam became the most defensive one.

2. **Swift concurrency / data-race warnings: 18 → 0** (`GooseAppModel+NotificationPipeline.swift` and collaborators), fixed at the root, not silenced.
   *Why it mattered:* the BLE notification pipeline mutates `@MainActor` UI/app state from inside background `DispatchQueue` closures. Under live packet ingestion that is a genuine **data race** — undefined behavior that corrupts UI state or crashes intermittently, and exactly the class of bug Swift 6 strict-concurrency refuses to ship. The fix gives each piece correct isolation: pure frame-interpretation is `nonisolated`; internally-synchronized services (`GooseBLEClient`, `PacketUIStateAggregator`, `WhoopDataSignalPipeline`, `HeartRateSeriesStore`, `CaptureFrameEnqueueAggregator`) are `@unchecked Sendable` with documented lock/serial-queue safety; queue-confined state is `nonisolated(unsafe)`. Off-main parsing stays off-main; only state mutation hops to the main actor. `xcodebuild -scheme GooseSwift` → **BUILD SUCCEEDED, 0 concurrency warnings**.

3. **Test suite: never-compiled → `cargo test --no-fail-fast` = 694 passed / 0 failed / 0 ignored.**
   *Why it mattered:* on a clean clone the suite **did not even compile** — one test pointed at a generated file via a path that resolved *above* the repo root, so `cargo test` failed instantly and **every contributor and any CI had zero test signal**. Unblocking compilation then surfaced real failures, which were fixed as **real bugs, not deletions** (see #4). The project now has a working, green test suite for the first time.

4. **Three real bugs the green suite uncovered:**
   - `import importlib` → `import importlib.util` in the Python reference adapters — `importlib.util.find_spec(...)` crashed *before* the hand-derived fallback could run, so the reference-algorithm tests failed even though the fallback was correct. *Why:* the reference adapters validate Goose's HRV/sleep math against known implementations — a broken adapter means no validation signal.
   - **Privacy-guard false positive** in `store.rs`/`export.rs` — the official-WHOOP-label guard rejected the metric's *own* policy-declaration string (`"official_whoop_values_are_validation_labels_not_inputs"`) because it began with `official_whoop_`. *Why:* that guard is the safety rail that keeps official WHOOP values from being treated as inputs; a false positive on its own config string would block legitimate local metrics while masking the real check.
   - **Wrong Swift-source path** in the iOS HealthKit-boundary tests (`<repo>/goose-swift/GooseSwift` vs the real `<repo>/GooseSwift`). *Why:* these tests enforce that the app reads **weight only** from HealthKit — a path bug meant that privacy boundary was never actually being checked.

5. **Clippy 120 → 0** on the library — real idiom fixes (`is_some()`+`unwrap()` → `if let`, `let-else` → `?`, manual `loop` → `while let`, `.max().min()` → `.clamp()`, collapsed duplicate `else-if`), never blanket suppressions.
   *Why it mattered:* several of these (notably `is_some()` then `unwrap()`) are real panic foot-guns, and a clean `clippy --all-targets -D warnings` is what keeps CI green and the next change honest.

6. **App Store compliance: 4 CRITICALs → GREENLIT.** Added `PrivacyInfo.xcprivacy` (Required-Reason API codes per Apple TN3183 — UserDefaults `CA92.1`, FileTimestamp `C617.1`), `CFBundleDisplayName`, cleared a false "tracking SDK" substring match, and wired the manifest into the target. `greenlight preflight .` → **GREENLIT, 0 CRITICAL**.
   *Why it mattered:* each CRITICAL is an automatic App Store rejection; the privacy manifest is **mandatory** for submission. Without these the app simply could not ship.

7. **Consolidated to a single `main`.** All work is committed and merged to `main`; the temporary `docs/goose-rescue-magazine` and leftover `codex-server` branches were removed so the repo is just one branch.

**Honestly still self-skipping** (not claimed as fully exercised): seven tests depend on artifacts that are the original author's private capture data or unauthored narrative docs, with no in-repo generator — the official-app→macOS-emulator command-evidence corpus (`fixtures/command-evidence/whoop-emulator-command-evidence.json`), the decompiled WHOOP-APK UI inventory (`apk-ui-inventory/coverage-map.json`), the example local-health validation manifest, and `Rust/docs/testing-and-tooling-strategy.md`. Those tests now read-and-skip-when-absent (with a clear skip message) rather than fail the suite, matching the existing precedent in `command_tests.rs`; they will exercise fully once the author supplies the artifacts. No fixture contents were fabricated. No upstream PR to `b-nnett/goose` has been opened.

## Project Layout

```text
GooseSwift/                         SwiftUI app source
GooseWorkoutLiveActivityExtension/  Live Activity widget extension
Rust/                               iOS static library, headers, per-platform outputs
Scripts/build_ios_rust.sh           Xcode build phase for the Goose Rust core
docs/goose-swift-mvp/               MVP plans, contracts, and data-readiness docs
GooseSwift.xcodeproj                Xcode project
```

Key Swift entry points:

- `GooseSwiftApp.swift`: app lifecycle and deep-link handling.
- `RootView.swift`: onboarding gate and global sync toast host.
- `AppShellView.swift`: tab shell and shared health store wiring.
- `GooseAppModel.swift`: app state, BLE ownership, lifecycle, and bridge summaries.
- `GooseBLEClient.swift`: Bluetooth scan/connect/sync logic.
- `GooseRustBridge.swift`: Swift wrapper around the Rust C bridge.
- `HealthView.swift` and `Health*` files: health dashboards, metric pages, trends, and sheets.
- `CoachView.swift` and `Coach*` files: coach UI and chat support.
- `MoreView.swift`: operational/debug/settings surfaces.

This is an active prototype. Because the data pipeline is still evolving, some metrics appear as empty or unavailable until the app has a source for them.

## Independence

Goose is an independent project and is not affiliated with WHOOP. This repository does not include or reference source code owned by WHOOP. The app communicates with WHOOP 5.0 bands over Bluetooth using services and data exposed by the device, then parses and stores that local data through the Goose Rust core. Product names are used only to describe compatibility.

## Design Credit

The current health metric UI draws heavily from [Bevel](https://www.bevel.health/), especially the Sleep, Recovery, Strain, Stress, and trend-detail surfaces. Bevel is not affiliated with Goose; this credit is here because their product design has been a major visual reference.

## Current Scope

- SwiftUI app shell with Home, Health, Coach, and More tabs.
- Onboarding and persisted profile state.
- CoreBluetooth scan/connect flows for WHOOP 5.0 devices.
- JSON-over-C bridge into the Goose Rust core.
- Health metric surfaces for Sleep, Recovery, Strain, Stress, Cardio Load, Energy Bank, Health Monitor, Packet Inputs, Algorithms, References, and Calibration.
- HealthKit sleep import and workout write support.
- Coach surfaces that summarize local metrics and explain missing data.
- More/Debug operational surfaces for device state, capture, sync, algorithms, storage, privacy, and support.
- Workout Live Activity extension.

## Requirements

- macOS with Xcode installed.
- iOS 26 SDK and an iOS 26 capable simulator/device.
- Apple Developer signing configured for the `com.goose.swift` bundle identifier.
- Rust and Cargo for building the Goose Rust core from the committed `Rust/core` source.
- iOS Rust targets installed with `rustup`; see the Rust Core Bridge section below.

Built Rust `.a` archives are generated locally during Xcode builds and are not committed. Set `GOOSE_SKIP_RUST_CORE_BUILD=1` only when the matching local archive already exists for the active Xcode platform.

## Build

Open `GooseSwift.xcodeproj` in Xcode and build the `GooseSwift` scheme, or build from the command line.

Simulator build:

```sh
xcodebuild \
  -project GooseSwift.xcodeproj \
  -scheme GooseSwift \
  -configuration Debug \
  -destination 'platform=iOS Simulator,name=iPhone 17' \
  -derivedDataPath /tmp/goose-swift-deriveddata \
  build
```

Physical device build:

```sh
xcodebuild \
  -project GooseSwift.xcodeproj \
  -scheme GooseSwift \
  -configuration Debug \
  -destination 'platform=iOS,id=<device-id>' \
  -derivedDataPath /tmp/goose-swift-deriveddata-device \
  -allowProvisioningUpdates \
  build
```

List connected devices:

```sh
xcrun devicectl list devices
```

## Reinstall On A Device

After a successful physical-device build, reinstall and launch:

```sh
xcrun devicectl device uninstall app \
  --device <device-id> \
  com.goose.swift

xcrun devicectl device install app \
  --device <device-id> \
  /tmp/goose-swift-deriveddata-device/Build/Products/Debug-iphoneos/GooseSwift.app

xcrun devicectl device process launch \
  --device <device-id> \
  --terminate-existing \
  com.goose.swift
```

## Rust Core Bridge

The Rust bridge source is committed in `Rust/core`. Do not commit built `.a`
archives; Xcode generates them locally through `Scripts/build_ios_rust.sh`.

Prerequisites:

- Xcode command line tools.
- Rust via `rustup`.
- iOS Rust targets:

```bash
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
```

`Scripts/build_ios_rust.sh` builds `Rust/core` for the active Xcode platform:

- `iphoneos` -> `aarch64-apple-ios`
- `iphonesimulator` on Apple Silicon -> `aarch64-apple-ios-sim`
- `iphonesimulator` on Intel -> `x86_64-apple-ios`

Outputs are staged into:

```text
Rust/iphoneos/libgoose_core.a
Rust/iphonesimulator/libgoose_core.a
```

The Swift target links `Rust/$(PLATFORM_NAME)/libgoose_core.a` and reads the C
bridge header from `Rust/core/include/goose_core_bridge.h`. The default Cargo
target directory is `build/rust-target/goose-core`, so Rust build products stay
outside the committed source tree.

Manual builds:

```bash
# Simulator on Apple Silicon
PLATFORM_NAME=iphonesimulator CURRENT_ARCH=arm64 Scripts/build_ios_rust.sh

# Physical iPhone
PLATFORM_NAME=iphoneos CURRENT_ARCH=arm64 Scripts/build_ios_rust.sh
```

You normally do not need to run these by hand; the Xcode build phase runs the
script before compiling Swift.

## Data And Privacy

- Metric views show empty, stale, or unavailable states when a source is missing.
- Metric rows and trend sheets show where values came from when that information is available.
- Raw packet payloads stay in debug/export flows rather than everyday health views.
- Coach responses use the same local metric summaries shown in the app.
- Health and fitness data is local by default. Any future backend or AI feature will need its own consent flow and privacy notes.

## Documentation

Detailed implementation plans live in `docs/goose-swift-mvp/`:

- `Home.md`: Home tab contract and remaining work.
- `Health.md`: Health surfaces, metric pages, packet inputs, trends, and acceptance checks.
- `Coach.md`: Coach tab plan and chat architecture notes.
- `More.md`: operational settings/debug/capture/privacy surfaces.
- `CodexCoachServer.md`: viability notes for a future Codex-powered coach.
- `RemainingDataTodo.md`: unresolved data-source and persistence work.

Recovery-specific follow-up work is tracked in `recovery-todo.md`.

## Contributing

This project moves quickly, so small focused changes are easiest to review.

Want to talk to other contributors? [Join the group here](https://x.com/i/chat/group_join/g2061785795330019536/3SHQtt2O8f).

- Keep changes close to the feature or bug you are working on.
- Match the existing SwiftUI style before introducing new patterns.
- Build after touching Swift, Rust bridge, project, or signing settings.
- Check both empty and populated states for metric UI when possible.
- Keep user-facing health copy plain and careful. Avoid medical claims.
- Put debug tooling, packet details, and raw export behavior under More or Debug surfaces.
- Update the relevant MVP doc when a change completes or changes an open task.
- Mention any build warnings, skipped checks, or device-only assumptions in the PR notes.

## Development Notes

- Prefer small, typed Swift models over displaying raw summary strings.
- Keep Home, Health, Coach, and More routes modular enough to work independently.
- Metric pages should still look polished when data is missing.
- Before installing to a device, run a simulator or device build and check that the Rust library target matches the destination platform.
