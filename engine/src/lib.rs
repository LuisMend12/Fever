//! Fever's audio engine.
//!
//! The JS ↔ Rust boundary is numbers only (no wasm-bindgen):
//! - `init(sample_rate)` once, after the AudioWorklet instantiates the module
//! - `trigger_kick()` when the UI sends a trigger message
//! - `render()` on every `process()` call, which fills the output buffer
//! - `output_ptr()` once, so JS can view the buffer as a `Float32Array`

mod kick;

use std::cell::UnsafeCell;

use kick::Kick;

/// Samples per render call. Matches the Web Audio render quantum.
pub const BLOCK_SIZE: usize = 128;

struct Engine {
    kick: Kick,
    /// Mono output. JS copies it to both stereo channels.
    output: [f32; BLOCK_SIZE],
}

impl Engine {
    const fn new() -> Self {
        Self {
            kick: Kick::new(),
            output: [0.0; BLOCK_SIZE],
        }
    }
}

/// Holds the global engine. `UnsafeCell` is Rust's marker for "mutated through a shared
/// reference"; we promise (below) that this only ever happens from one thread.
struct EngineCell(UnsafeCell<Engine>);

// SAFETY: this crate is only run as a WASM module inside one AudioWorklet, which is a single
// thread. Nothing ever accesses the cell from two threads.
unsafe impl Sync for EngineCell {}

/// The one engine instance. Rust owns this memory; JS only reads `output` through a pointer.
static ENGINE: EngineCell = EngineCell(UnsafeCell::new(Engine::new()));

fn engine() -> &'static mut Engine {
    // SAFETY: single thread (see `EngineCell`), and each export takes this reference once and
    // drops it before returning. The exports never call each other, so at most one
    // `&mut Engine` exists at a time.
    unsafe { &mut *ENGINE.0.get() }
}

#[unsafe(no_mangle)]
pub extern "C" fn init(sample_rate: f32) {
    engine().kick.set_sample_rate(sample_rate);
}

#[unsafe(no_mangle)]
pub extern "C" fn trigger_kick() {
    engine().kick.trigger();
}

#[unsafe(no_mangle)]
pub extern "C" fn render() {
    let engine = engine();
    engine.kick.render(&mut engine.output);
}

/// Byte offset of the output buffer inside WASM linear memory.
#[unsafe(no_mangle)]
pub extern "C" fn output_ptr() -> *const f32 {
    engine().output.as_ptr()
}

/// Length of the output buffer, so JS doesn't have to hard-code it.
#[unsafe(no_mangle)]
pub extern "C" fn block_size() -> u32 {
    BLOCK_SIZE as u32
}
