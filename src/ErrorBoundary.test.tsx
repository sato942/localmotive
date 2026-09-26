// @vitest-environment jsdom
//
// Error-boundary reset contract (FE-08): the reset must reload even when
// storage access throws, instead of aborting on the error screen. The
// reload is injectable so the test never navigates the jsdom window.
import { afterEach, describe, expect, it, vi } from "vitest";
import { resetSavedState } from "./ErrorBoundary";

describe("error boundary reset (FE-08)", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
    window.localStorage.clear();
  });

  it("reloads even when storage access throws", () => {
    // jsdom storage does not route through Storage.prototype, so replace
    // the binding with a denied store instead of spying on the prototype.
    vi.stubGlobal("localStorage", {
      get length() {
        return 1;
      },
      key() {
        throw new Error("denied");
      },
      removeItem() {},
    });
    const reload = vi.fn();

    expect(() => resetSavedState(reload)).not.toThrow();
    expect(reload).toHaveBeenCalledTimes(1);
  });

  it("clears application keys, keeps quarantine and foreign keys, and reloads", () => {
    window.localStorage.setItem("localmotive:theme", "dark");
    window.localStorage.setItem("localmotive:quarantine:x", "kept");
    window.localStorage.setItem("other", "kept");
    const reload = vi.fn();

    resetSavedState(reload);

    expect(reload).toHaveBeenCalledTimes(1);
    expect(window.localStorage.getItem("localmotive:theme")).toBe(null);
    expect(window.localStorage.getItem("localmotive:quarantine:x")).toBe("kept");
    expect(window.localStorage.getItem("other")).toBe("kept");
  });
});
