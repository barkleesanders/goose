# 🪿 The Goose Rescue

> A field report on dragging an AI-generated app to ship-ready.
> *Engineering Edition № 001 · June 3, 2026*

**📄 Read the designed edition:**
- **[goose-rescue.pdf](./goose-rescue.pdf)** — the full magazine, print/PDF (7 spreads)
- **[goose-rescue.html](./goose-rescue.html)** — the interactive HTML edition (open in a browser)

187,000 lines written by agents in two days. Good bones, dangerous habits. Here is what a human-directed pass with `/carmack`, `/debug`, and `/ios-ship` did to it.

---

## 01 · The Patient

Goose arrived as a WHOOP-style recovery tracker: a **fat Rust core** doing the math, a **thin SwiftUI** skin on top, and a hand-rolled **JSON FFI bridge** stitching the two together. Ten commits across two days — one literally read *“Merge branch 'codex-server'.”* Nobody had ever run it.

The architecture was genuinely good — clean separation, exhaustive tests. But it carried the classic tells of machine-written code:

| | |
|---|---|
| **187K** | lines of code |
| **10** | commits / 2 days |
| **0** | humans had run it |
| **120** | clippy warnings |

An FFI boundary that turned any Rust hiccup into an instant crash, 120 unaddressed lints, 8,000-line files, no Apple privacy manifest, and test fixtures never committed. Clever, untested, and one panic away from a black screen.

## 02 · Fix №1 — Don't let the core take the whole app down

The single highest-impact change. The FFI seam was set to `panic = "abort"`, so **any** panic in the Rust core became a `SIGABRT` — the app died on screen.

```rust
// BEFORE — a panic in Rust = SIGABRT = the app dies
panic = "abort"        // Cargo.toml

// AFTER — panics are caught and returned as structured errors
panic = "unwind"       // Cargo.toml

#[no_mangle]
pub extern "C" fn goose_bridge_handle_json(req: *const c_char) -> *mut c_char {
    match catch_unwind(AssertUnwindSafe(|| handle_bridge_request_json(req))) {
        Ok(resp) => resp,
        Err(_)   => bridge_error("core_panic"),   // ← app survives
    }
}
```

Both `extern "C"` entry points now sit behind `catch_unwind`. A crash in the math layer becomes a JSON error the Swift side can show — not a dead app. The app's most fragile seam is now its most defensive one.

## 03 · The Lint Ledger — 120 → 0

Clippy warnings on the library taken to **zero** — real fixes, not blanket suppressions:

- `is_some()` + `unwrap()` → `if let Some(..)`
- `let-else` → the `?` operator (×4)
- manual `loop` → `while let`
- `.max(0.).min(180.)` → `.clamp(0., 180.)`
- collapsed duplicate `else-if` blocks
- one `.expect()` **kept** — its bounds invariant was proven, and the FFI `catch_unwind` now guards it anyway

## 04 · App Store — four CRITICALs → zero

`greenlight preflight .` flagged four blockers. All fixed, no fabricated declarations:

- ✕→✓ **Missing privacy manifest** → `PrivacyInfo.xcprivacy` (UserDefaults `CA92.1`, FileTimestamp `C617.1` per Apple TN3183)
- ✕→✓ **No display name** → `CFBundleDisplayName` “Goose”
- ✕→✓ **False “Amplitude SDK” match** → renamed a waveform variable; verified no tracking SDK exists
- ✕→✓ **Manifest not in target** → wired in via `pbxproj`, confirmed bundled in the `.app`

**Result: GREENLIT — 0 CRITICAL.**

## 05 · It runs

Downloaded the **8.52 GB iOS 26.5** simulator runtime (`23F77`), rebuilt, and watched it boot.

- **BUILD SUCCEEDED** — iOS 26.5
- **PID 50947** — launched, onboarding (“Import Weight”) live
- **clean** — no crash, no FFI error in the logs

## 06 · What we did *not* pretend was done

Shipping-ready is not finished. The reference frame is the full spec, not the distance travelled — so the open items stay named:

- **16 Swift concurrency / data-race warnings** — 15 clustered in `GooseAppModel+NotificationPipeline.swift`, 1 in `HealthDataStore.swift`. The real remaining bug surface.
- **45 test-code clippy warnings** + uncommitted test fixtures — the full suite can't run until those are generated.
- **Nothing pushed upstream.** No PR to `b-nnett/goose` — by rule, awaiting approval. (This branch lives on a fork.)

> A green build that crashes on launch is not done. This one launched clean — but the races are real and unfixed, and we said so.

---

*Rendered by the `/magazine` skill. Every figure above is from the actual `/carmack` + `/debug` + `/ios-ship` work on this repo — nothing invented. The Rust/Swift source changes themselves remain local pending review; this branch carries only the write-up.*
