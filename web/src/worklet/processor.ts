// Runs on the audio thread. Owns the WASM instance and copies the engine's output into the
// browser's buffers on every process() call.

import { PROCESSOR_NAME, type FromWorklet, type ToWorklet } from "../messages";

/** Functions exported by engine/src/lib.rs. */
interface EngineExports {
  memory: WebAssembly.Memory;
  init(sampleRate: number): void;
  trigger_kick(): void;
  render(): void;
  output_ptr(): number;
  block_size(): number;
}

class FeverProcessor extends AudioWorkletProcessor {
  private engine: EngineExports | null = null;
  private output: Float32Array | null = null;

  constructor() {
    super();
    this.port.onmessage = (event: MessageEvent<ToWorklet>) => this.handleMessage(event.data);
  }

  private handleMessage(message: ToWorklet): void {
    switch (message.type) {
      case "load":
        this.load(message.module);
        break;
      case "trigger":
        this.engine?.trigger_kick();
        break;
    }
  }

  private load(module: WebAssembly.Module): void {
    try {
      const instance = new WebAssembly.Instance(module, {});
      const engine = instance.exports as unknown as EngineExports;
      engine.init(sampleRate);
      this.engine = engine;
      this.output = this.viewOutput(engine);
      this.send({ type: "ready" });
    } catch (error) {
      this.send({ type: "error", message: error instanceof Error ? error.message : String(error) });
    }
  }

  /** A Float32Array that looks directly at Rust's output buffer inside WASM memory. */
  private viewOutput(engine: EngineExports): Float32Array {
    return new Float32Array(engine.memory.buffer, engine.output_ptr(), engine.block_size());
  }

  private send(message: FromWorklet): void {
    this.port.postMessage(message);
  }

  process(_inputs: Float32Array[][], outputs: Float32Array[][]): boolean {
    const engine = this.engine;
    if (!engine || !this.output) {
      return true; // Not loaded yet: the browser's buffers are already silent.
    }

    engine.render();

    // If WASM memory ever grows, the old buffer is detached and the view's length drops to 0.
    if (this.output.length === 0) {
      this.output = this.viewOutput(engine);
    }

    // Rust renders mono; copy it to every output channel (left and right).
    // `set` copies without allocating, which matters on the audio thread (no GC pauses).
    // The engine renders exactly one 128-sample block, the Web Audio default.
    for (const channel of outputs[0]) {
      if (channel.length === this.output.length) {
        channel.set(this.output);
      }
    }
    return true;
  }
}

registerProcessor(PROCESSOR_NAME, FeverProcessor);
