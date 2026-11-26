// Messages sent between the main thread and the AudioWorklet over its MessagePort.
// Shared by both sides so they agree on the shape.

/** Main thread → worklet. */
export type ToWorklet =
  | { type: "load"; module: WebAssembly.Module }
  | { type: "trigger" };

/** Worklet → main thread. */
export type FromWorklet = { type: "ready" } | { type: "error"; message: string };

/** Name the processor is registered under. */
export const PROCESSOR_NAME = "fever-engine";
