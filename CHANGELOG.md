# Changelog

All notable changes to Fever are recorded here, newest first.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Each entry links to the issue it belongs to. Decisions and their reasons live in the linked issues.

## [Unreleased]

### Added
- **First sound: a synthesized kick drum** ([#2](https://github.com/LuisMend12/Fever/issues/2)).
  Press **Space** or click the **Kick** pad. The first press starts audio, since browsers require a user gesture.
- **Rust audio engine** in `engine/`, compiled with plain `cargo build --target wasm32-unknown-unknown` (no `wasm-bindgen`).
  - Exports over a numbers-only boundary: `init(sample_rate)`, `trigger_kick()`, `render()`, `output_ptr()`, `block_size()`.
  - The engine owns a fixed 128-sample output buffer. JS reads it through a `Float32Array` view at `output_ptr()`.
  - Kick sound: a sine wave sweeping from 150 Hz toward 50 Hz with an exponential volume decay, 0.4 s long. Mono, copied to both stereo channels.
  - 7 unit tests (`npm test`) covering silence before a trigger, sound after it, ±1 range, decay to silence, retrigger, and invalid sample rates.
- **AudioWorklet loading** (`web/src/audio.ts`, `web/src/worklet/processor.ts`). The main thread fetches and compiles the `.wasm` and posts the module to the worklet, which instantiates it and replies `ready`. Startup gives up after 5 s and shows an error.
- **UI** in Solid 1.9 + TypeScript (`web/src/App.tsx`) with a status line and the Kick pad.
- **Tooling:** Parcel 2 (with `packageExports` enabled so Solid's browser build is used), `babel-preset-solid` via `.babelrc`, `tsconfig.json`, and npm scripts `dev`, `build`, `test`, `typecheck`, `build:engine`.
- **Docs:** README sections for planned features, roadmap, architecture, getting started, project layout, contributing, and sample licensing.
- **Project tracking:** GitHub issues #1–#10 for decisions and each build step.

### Decided
- Stack, bundler, WASM build, first sound, and backend. See [#1](https://github.com/LuisMend12/Fever/issues/1) (closed).
- Solid **1.9** rather than 2.0, because Solid 2.0's compiler preset is still a release candidate.

### Known issues
- Retriggering the kick while it's still sounding restarts it instantly, which can click. Fade vs. overlapping hits is still to be decided ([#4](https://github.com/LuisMend12/Fever/issues/4)).
- The `AudioContext` is never closed (fine for now, since the app never unmounts).
- The worklet assumes the default 128-sample render block.

### Setup notes
- Windows without Visual Studio Build Tools: install Rust with the GNU host toolchain (`rustup-init --default-host x86_64-pc-windows-gnu`) so `cargo test` can link.
