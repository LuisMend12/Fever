// TypeScript's DOM types don't include the AudioWorklet global scope, so we declare the parts
// the processor uses.

declare const sampleRate: number;

declare class AudioWorkletProcessor {
  readonly port: MessagePort;
  constructor();
}

declare function registerProcessor(
  name: string,
  processorCtor: new () => AudioWorkletProcessor,
): void;
