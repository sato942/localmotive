/// <reference types="vite/client" />

declare global {
  interface Window {
    /**
     * Set by the Tauri runtime in the packaged app and the dev shell.
     * Absent in plain browsers and in tests unless a fixture installs it.
     */
    __TAURI_INTERNALS__?: unknown;
  }
}

export {};
