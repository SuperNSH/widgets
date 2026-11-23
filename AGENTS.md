# AGENTS.md

Hard rules for humans and coding agents in this repo.

- The main branch is `master`.
- Commit small changes with a short conventional message. No attribution lines.
- Comments are 1-2 lines and state a non-obvious constraint.
- Docs read like a competent human wrote them: direct, concrete, no marketing filler.
- Every behaviour change needs a test. A test is seen to fail once before it counts.
- Local simulator and emulator runs are allowed here, because a widget needs a home screen to be seen. Use simulators and AVDs named after this repo (for example `nsh-widgets-iphone`). Never erase, shut down or kill a simulator, emulator or process you did not create.
- Status claims in README and docs need evidence under `docs/evidence/`: a screenshot someone looked at, or a command and its output. A claim with no evidence is marked unverified.
- The renderers stay generic. No widget-specific code in `NshWidgets.swift` or the Java renderer; a widget is data, never a native class.
- The kit (`packages/nsh-widgets/src/kit.ts`) compiles only to base nodes. A kit change never needs a renderer change.
- An IR change lands in four places together: the TS types (`base.ts`), the check in `crates/nsh-widgets/src/store.rs`, the Swift renderer and the Java renderer. Update [docs/ir.md](docs/ir.md) and [docs/capabilities.md](docs/capabilities.md) with it.
- After changing the Java sources, layouts or drawables, rebuild `crates/nsh-widgets/android/nsh-widgets.aar` with `scripts/build-aar.sh`. The AAR is committed.

## Checks

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test -p nsh-widgets`
- in `packages/nsh-widgets`: `nsh check`, `nsh test`
- for a change that reaches a renderer: build the gallery with `nsh-widgets build`, package it, place the widget, and save a screenshot under `docs/evidence/<platform>/`
