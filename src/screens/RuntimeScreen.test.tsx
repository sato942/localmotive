// @vitest-environment jsdom
//
// Contract tests for the extracted Runtime screen (audit S-27.I2): the
// component is presentational, so these tests hand it fixture props and
// assert the rendered consequences a user would notice. They pin the parts
// the extraction could silently break: the install identity surfaced for a
// managed runtime (RT-06), the empty catalog message, and the health-cancel
// pending state.
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { RuntimeScreen, type RuntimeScreenProps } from "./RuntimeScreen";
import type { ManagedRuntimeRecord } from "../model";

let container: HTMLDivElement;
let root: Root;

beforeEach(() => {
  container = document.createElement("div");
  document.body.appendChild(container);
  root = createRoot(container);
});

afterEach(() => {
  act(() => root.unmount());
  container.remove();
});

function props(overrides: Partial<RuntimeScreenProps> = {}): RuntimeScreenProps {
  const record: ManagedRuntimeRecord = {
    tag: "b6100",
    backend: "cuda",
    installKey: "cuda-12.4-windows-x64",
    runtimePath: "C:/Localmotive/runtimes/b6100/llama-server.exe",
    installRoot: "C:/Localmotive/runtimes/b6100",
    contentVerified: true,
  };
  return {
    activateRuntime: () => {},
    busy: "",
    cancelHealthRepair: () => {},
    cancelManagedHealth: () => {},
    cancelRuntimeInstall: () => {},
    chooseExistingRuntime: () => {},
    hardware: null,
    healthCancelling: false,
    healthModelProgress: null,
    healthRepairNotice: null,
    healthRepairing: false,
    healthResult: null,
    healthRunning: "",
    inspect: () => {},
    installRuntime: () => {},
    installing: "",
    isManagedPath: false,
    loadRuntimeSetup: () => {},
    managedRuntimes: [record],
    modelRoot: "C:/models",
    openExternal: () => {},
    repairHealthModel: () => {},
    runManagedHealth: () => {},
    runtime: null,
    runtimeCatalog: null,
    runtimeCatalogLoading: false,
    runtimeCatalogState: { kind: "idle" },
    runtimeIdentity: null,
    runtimeInstallCancelling: false,
    runtimeInstallProgress: null,
    runtimePath: "C:/Localmotive/runtimes/b6100/llama-server.exe",
    runtimeRoot: "C:/Localmotive/runtimes",
    selectRuntimeAdapter: () => {},
    selectedRuntimeAdapterId: "",
    setRuntime: () => {},
    setRuntimePath: () => {},
    ...overrides,
  };
}

describe("RuntimeScreen presentation contract", () => {
  it("shows the managed install identity for an installed runtime", () => {
    act(() => root.render(<RuntimeScreen {...props()} />));
    const text = container.textContent ?? "";
    expect(text).toContain("Runtime manager");
    expect(text).toContain("cuda-12.4-windows-x64");
  });

  it("shows the empty catalog message without a loading indicator", () => {
    act(() =>
      root.render(
        <RuntimeScreen
          {...props({
            runtimeCatalogState: {
              kind: "empty",
              catalog: {
                tag: "b6100",
                publishedAt: "2026-09-01T00:00:00Z",
                options: [],
                availability: [],
                origin: "cache",
              } as RuntimeScreenProps["runtimeCatalogState"] extends { catalog: infer C }
                ? C
                : never,
            },
          })}
        />,
      ),
    );
    const text = container.textContent ?? "";
    expect(text).toContain("No approved runtime matches this system.");
    expect(container.querySelectorAll(".runtime-loading").length).toBe(0);
  });

  it("offers cancellation while a managed health run is active", () => {
    act(() => root.render(<RuntimeScreen {...props({ healthRunning: "cuda-12.4-windows-x64" })} />));
    const buttons = Array.from(container.querySelectorAll("button"));
    const cancel = buttons.find((b) => (b.textContent ?? "").includes("Cancel and clean up"));
    expect(cancel).toBeTruthy();
    expect(cancel!.disabled).toBe(false);
  });
});
