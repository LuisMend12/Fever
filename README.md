# Fever

An open-source digital audio workstation (DAW) that runs in the browser. Fever uses a pattern-based workflow: build beats and melodies in patterns, then arrange them into songs. All of its instruments and effects are written from scratch, so it needs no third-party plugins.

> **Status:** early planning. Nothing is playable yet. Follow progress in the [issues](https://github.com/LuisMend12/Fever/issues).

## Planned features

- **Instruments:** guitar and drums to start, each available as a synthesizer (sound generated in code) and as a sample player (recorded audio)
- **Live playing:** computer keyboard and on-screen pads
- **Transport:** play, stop, tempo, and a sample-accurate clock
- **Step sequencer:** a grid for building beats quickly
- **Piano roll:** a note editor for melodies and chords
- **Playlist:** arrange patterns into a full song
- **Mixer and effects:** per-instrument channels with built-in effects
- **Save / load:** keep projects and open them later

## Roadmap

Features are built in this order, since each step depends on the ones before it:

| Step | Feature | Issue |
|---|---|---|
| 0 | Decide build tooling and first sound | [#1](https://github.com/LuisMend12/Fever/issues/1) |
| 1 | Project scaffold + first sound | [#2](https://github.com/LuisMend12/Fever/issues/2) |
| — | Collect CC0 samples | [#3](https://github.com/LuisMend12/Fever/issues/3) |
| 2 | Instruments: guitar and drums | [#4](https://github.com/LuisMend12/Fever/issues/4) |
| 3 | Transport and clock | [#5](https://github.com/LuisMend12/Fever/issues/5) |
| 4 | Step sequencer | [#6](https://github.com/LuisMend12/Fever/issues/6) |
| 5 | Piano roll | [#7](https://github.com/LuisMend12/Fever/issues/7) |
| 6 | Playlist / arrangement | [#8](https://github.com/LuisMend12/Fever/issues/8) |
| 7 | Mixer and effects | [#9](https://github.com/LuisMend12/Fever/issues/9) |
| 8 | Save / load projects | [#10](https://github.com/LuisMend12/Fever/issues/10) |

## Architecture (planned)

```
 Browser
 ┌──────────────────────────────┐        ┌───────────────────────────────┐
 │ Main thread                  │        │ AudioWorklet (audio thread)   │
 │ UI: TypeScript + Solid       │ ─────► │ Audio engine: Rust → WASM     │
 │ keys, pads, editors          │ ◄───── │ instruments, effects, mixing  │
 └──────────────────────────────┘        └───────────────┬───────────────┘
                                                         ▼
                                                      speakers
```

- **Audio engine:** Rust compiled to WebAssembly, running inside a Web Audio `AudioWorklet`. The browser asks it for a small block of audio (128 samples by default) every few milliseconds, so it has to be fast and predictable.
- **UI:** TypeScript with [Solid](https://www.solidjs.com/), running on the main thread. It sends commands such as "play this note" to the engine.

- **Bundler:** [Parcel](https://parceljs.org/), which can bundle the AudioWorklet directly
- **WASM build:** plain `cargo build --target wasm32-unknown-unknown` with hand-written `extern "C"` exports (no `wasm-bindgen`)
- **Backend:** none for now. One will be added if a feature needs it.

Decided in [#1](https://github.com/LuisMend12/Fever/issues/1). Still open: where the sequencer clock lives and who owns project state ([#5](https://github.com/LuisMend12/Fever/issues/5)).

## Getting started

Right now Fever plays one synthesized kick drum: press **Space** or click the **Kick** pad.

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable), plus the WebAssembly target:
  ```sh
  rustup target add wasm32-unknown-unknown
  ```
  On Windows without Visual Studio Build Tools, install the GNU host toolchain
  (`rustup-init --default-host x86_64-pc-windows-gnu`) so `cargo test` can link.
- [Node.js](https://nodejs.org/) 20 or newer
- A modern browser (tested in Chrome)

### Run it

```sh
npm install
npm run dev      # builds the Rust engine, then serves the app at http://localhost:1234
```

The first Space press or pad click starts audio. Browsers don't allow sound before a user gesture.

### Other commands

| Command | What it does |
|---|---|
| `npm run build` | Production build into `dist/` |
| `npm test` | Rust unit tests for the engine |
| `npm run typecheck` | TypeScript type check |
| `npm run build:engine` | Rebuild only the `.wasm` engine |

`npm run dev` builds the engine once at startup. After changing Rust code, run `npm run build:engine` in another terminal, then reload the page.

### Project layout

```
engine/            Rust audio engine (compiled to WebAssembly)
  src/lib.rs       exported functions: init, trigger_kick, render, output_ptr, block_size
  src/kick.rs      the kick drum synth + its tests
web/
  index.html
  src/audio.ts     creates the AudioContext, compiles the .wasm, loads the worklet
  src/worklet/     AudioWorkletProcessor that runs the engine on the audio thread
  src/messages.ts  message types shared by the UI and the worklet
  src/App.tsx      Solid UI: Space key + Kick pad
```

## Contributing

1. Pick an open [issue](https://github.com/LuisMend12/Fever/issues), or open one to discuss a new idea.
2. Create a branch from `main`.
3. Open a pull request that references the issue (for example `Closes #4`).

Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:`, `chore:` ...).

## Samples and licensing

The code is licensed under [MIT](LICENSE). Audio samples in this repo keep their own licenses:

- Only commit samples that are **CC0** (public domain), or **CC-BY** with credit given.
- Do **not** commit CC-BY-NC, CC-BY-ND, or commercial "royalty-free" packs. They don't allow redistribution here.
- Record each sample's source URL, author, and license next to the files.

## License

[MIT](LICENSE)
