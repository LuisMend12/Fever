import { createSignal, onCleanup, onMount, Show } from "solid-js";
import { startAudioEngine, type AudioEngine } from "./audio";

type Status = "idle" | "starting" | "ready" | "error";

const KICK_KEY = "Space";

export function App() {
  const [status, setStatus] = createSignal<Status>("idle");
  const [errorMessage, setErrorMessage] = createSignal("");
  const [hit, setHit] = createSignal(false);

  // Started on the first key press or pad click, then reused.
  let engine: Promise<AudioEngine> | null = null;

  const getEngine = (): Promise<AudioEngine> => {
    if (!engine) {
      setStatus("starting");
      engine = startAudioEngine().then(
        (started) => {
          setStatus("ready");
          return started;
        },
        (error: unknown) => {
          engine = null; // Let the next press try again.
          setErrorMessage(error instanceof Error ? error.message : String(error));
          setStatus("error");
          throw error;
        },
      );
    }
    return engine;
  };

  const playKick = () => {
    flashPad();
    // A failure is already shown in the UI by getEngine.
    getEngine().then((started) => started.triggerKick(), () => {});
  };

  let flashTimer: number | undefined;
  const flashPad = () => {
    setHit(true);
    clearTimeout(flashTimer);
    flashTimer = window.setTimeout(() => setHit(false), 120);
  };

  const onKeyDown = (event: KeyboardEvent) => {
    if (event.code !== KICK_KEY || event.repeat) {
      return;
    }
    event.preventDefault(); // Stop Space from scrolling the page.
    playKick();
  };

  onMount(() => window.addEventListener("keydown", onKeyDown));
  onCleanup(() => {
    window.removeEventListener("keydown", onKeyDown);
    clearTimeout(flashTimer);
  });

  return (
    <main class="app">
      <header>
        <h1>Fever</h1>
        <p class="status" data-status={status()}>
          {statusText(status())}
        </p>
      </header>

      <section class="pads">
        {/* pointerdown instead of click: drums should sound on press, not on release. */}
        <button
          type="button"
          class="pad"
          classList={{ hit: hit() }}
          onPointerDown={playKick}
          aria-label="Kick (Space)"
        >
          Kick
          <span class="key">Space</span>
        </button>
      </section>

      <Show when={status() === "error"}>
        <p class="error" role="alert">
          {errorMessage()}
        </p>
      </Show>
    </main>
  );
}

function statusText(status: Status): string {
  switch (status) {
    case "idle":
      return "Press Space or click the pad to start audio";
    case "starting":
      return "Starting audio…";
    case "ready":
      return "Audio on";
    case "error":
      return "Audio failed to start";
  }
}
