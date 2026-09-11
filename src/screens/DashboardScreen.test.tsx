// @vitest-environment jsdom
//
// FE-16.V3 behavior pins for the Control screen: the log well retains the
// last bounded output after the server stops or exits, falls back to the
// empty state when no output was ever captured, and never leaks refactor
// placeholders into user copy.
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { DashboardScreen, type DashboardScreenProps } from "./DashboardScreen";
import type { ServerStatus } from "../model";

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

function status(overrides: Partial<ServerStatus> = {}): ServerStatus {
  return {
    running: false,
    phase: "idle",
    pid: null,
    profileName: null,
    alias: null,
    port: null,
    command: null,
    logPath: "C:/Localmotive/logs/run.log",
    startedAt: null,
    exitCode: null,
    specType: "draft-none",
    companionLinked: false,
    resultClass: "unknown",
    validation: null,
    failure: null,
    ...overrides,
  };
}

function props(overrides: Partial<DashboardScreenProps> = {}): DashboardScreenProps {
  return {
    benchmark: null,
    busy: "",
    evidenceRun: null,
    log: "",
    openExternal: () => {},
    profile: null,
    runtime: null,
    selected: undefined,
    setView: () => {},
    start: () => {},
    status: status(),
    stop: () => {},
    tuning: false,
    ...overrides,
  };
}

function render(overrides: Partial<DashboardScreenProps> = {}) {
  act(() => {
    root.render(<DashboardScreen {...props(overrides)} />);
  });
  return container;
}

describe("DashboardScreen log well (FE-16)", () => {
  it("keeps the last bounded output visible with a stopped note after an exit", () => {
    const tail = "llama_server: server stopped unexpectedly\nexit code 1";
    const view = render({
      log: tail,
      status: status({ exitCode: 1, phase: "idle", running: false }),
    });
    const retained = view.querySelector(".log-retained");
    expect(retained).not.toBeNull();
    expect(retained?.textContent).toContain("Server is stopped.");
    expect(retained?.textContent).toContain("The last bounded output remains visible until the next start.");
    expect(retained?.querySelector("pre")?.textContent).toContain("exit code 1");
  });

  it("streams the live output while the server runs", () => {
    const view = render({
      log: "llama_server: listening on 127.0.0.1:8080",
      status: status({ running: true, phase: "running", pid: 4242, port: 8080 }),
    });
    expect(view.querySelector(".log-retained")).toBeNull();
    const pre = view.querySelector(".terminal-panel pre");
    expect(pre?.textContent).toContain("listening on 127.0.0.1:8080");
  });

  it("shows the empty state with the corrected guidance text when no output exists yet", () => {
    const view = render({ log: "" });
    expect(view.querySelector(".log-retained")).toBeNull();
    const empty = view.querySelector(".log-empty");
    expect(empty).not.toBeNull();
    // The refactor placeholder regression: this text rendered "props.profile".
    expect(empty?.textContent).toContain("Start this profile to stream llama-server output here.");
    expect(empty?.textContent).not.toContain("props.");
  });
});
