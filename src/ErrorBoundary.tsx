import { Component, type ErrorInfo, type ReactNode } from "react";

// Audit FE-09: a render error anywhere below this boundary must never leave
// a blank window. The fallback names the failure, keeps the raw records for
// diagnosis, and offers a reset that clears only this application's keys.
type ErrorBoundaryState = { error: string | null };

function clearApplicationStorage() {
  const doomed: string[] = [];
  for (let index = 0; index < localStorage.length; index += 1) {
    const key = localStorage.key(index);
    if (!key) continue;
    const isApplicationKey =
      (key.startsWith("localmotive:") || key.startsWith("gguf-pilot:")) &&
      !key.startsWith("localmotive:quarantine:");
    if (isApplicationKey) doomed.push(key);
  }
  for (const key of doomed) {
    localStorage.removeItem(key);
  }
}

/// Reset the saved state and reload: storage cleanup must never block the
/// reload, so a denied-storage failure still leaves the error screen
/// instead of throwing a second UI error. The reload is injectable for tests.
export function resetSavedState(reload: () => void = () => window.location.reload()) {
  try {
    clearApplicationStorage();
  } catch (error) {
    // The reload below still runs: report the leftover keys instead of
    // throwing a second UI error from the reset button.
    console.error("Localmotive could not clear saved state before reload", error);
  } finally {
    reload();
  }
}

export class ErrorBoundary extends Component<{ children: ReactNode }, ErrorBoundaryState> {
  state: ErrorBoundaryState = { error: null };

  static getDerivedStateFromError(error: unknown): ErrorBoundaryState {
    return { error: error instanceof Error ? error.message : String(error) };
  }

  componentDidCatch(error: unknown, info: ErrorInfo) {
    console.error("Localmotive UI error boundary", error, info.componentStack);
  }

  render() {
    if (this.state.error === null) {
      return this.props.children;
    }
    return (
      <main className="screen" role="alert">
        <h1>Localmotive hit a display error</h1>
        <p>
          The window could not render this screen. Unreadable saved records are kept under
          <code> localmotive:quarantine:*</code> keys for diagnosis.
        </p>
        <p className="mono">{this.state.error}</p>
        <button
          type="button"
          className="button secondary"
          onClick={() => {
            resetSavedState();
          }}
        >
          Reset saved state and reload
        </button>
      </main>
    );
  }
}
