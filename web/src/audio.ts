// Main-thread side of the audio engine: creates the AudioContext, compiles the WASM, and loads
// it into the worklet.

import workletUrl from "worklet:./worklet/processor.ts";
import { PROCESSOR_NAME, type FromWorklet, type ToWorklet } from "./messages";

const wasmUrl = new URL(
  "../../engine/target/wasm32-unknown-unknown/release/fever_engine.wasm",
  import.meta.url,
);

export interface AudioEngine {
  triggerKick(): void;
}

/**
 * Must be called from a user gesture (click or key press): browsers only let audio start
 * after the user interacts with the page.
 */
export async function startAudioEngine(): Promise<AudioEngine> {
  // Created synchronously, inside the gesture, so the browser allows it to run.
  const context = new AudioContext();

  try {
    const [module] = await Promise.all([
      compileWasm(),
      context.audioWorklet.addModule(workletUrl),
    ]);

    const node = new AudioWorkletNode(context, PROCESSOR_NAME, {
      numberOfInputs: 0,
      numberOfOutputs: 1,
      outputChannelCount: [2],
    });
    node.connect(context.destination);

    // Resume before waiting: some browsers don't run the worklet until the context is running.
    if (context.state === "suspended") {
      await context.resume();
    }
    await loadIntoWorklet(node, module);

    const send = (message: ToWorklet) => node.port.postMessage(message);
    return { triggerKick: () => send({ type: "trigger" }) };
  } catch (error) {
    await context.close();
    throw error;
  }
}

async function compileWasm(): Promise<WebAssembly.Module> {
  const response = await fetch(wasmUrl);
  if (!response.ok) {
    throw new Error(`Could not load the audio engine (HTTP ${response.status}).`);
  }
  return WebAssembly.compile(await response.arrayBuffer());
}

/** How long to wait for the worklet to report ready before giving up. */
const READY_TIMEOUT_MS = 5000;

/** Sends the compiled module to the worklet and waits until it reports ready. */
function loadIntoWorklet(node: AudioWorkletNode, module: WebAssembly.Module): Promise<void> {
  return new Promise((resolve, reject) => {
    const fail = (reason: string) => {
      clearTimeout(timer);
      reject(new Error(`Audio engine failed to start: ${reason}`));
    };
    const timer = setTimeout(() => fail("no response from the audio thread"), READY_TIMEOUT_MS);

    node.onprocessorerror = () => fail("the audio processor crashed");
    node.port.onmessage = (event: MessageEvent<FromWorklet>) => {
      const reply = event.data;
      if (reply.type === "ready") {
        clearTimeout(timer);
        resolve();
      } else {
        fail(reply.message);
      }
    };
    const message: ToWorklet = { type: "load", module };
    node.port.postMessage(message);
  });
}
