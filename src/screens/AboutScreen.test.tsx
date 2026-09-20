// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { expect, it, vi } from "vitest";
import { AboutScreen } from "./AboutScreen";

it("opens manual releases and icon credits through existing allowed GitHub URLs", () => {
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  const container = document.createElement("div");
  const root = createRoot(container);
  const open = vi.fn();
  try {
    act(() => root.render(<AboutScreen about={null} models={[]} totalBytes={0} runtime={null} runtimeIdentity={null} hardware={null} modelRoot="" onOpenExternal={open} />));
    const button = (label: string) => [...container.querySelectorAll("button")].find((item) => item.textContent?.includes(label));
    expect(button("Check releases")).toBeDefined();
    act(() => button("Check releases")!.click());
    expect(open).toHaveBeenLastCalledWith("https://github.com/sato942/localmotive/releases/latest");
    act(() => button("Lucide")!.click());
    expect(open).toHaveBeenLastCalledWith("https://github.com/lucide-icons/lucide");
    expect(container.textContent).toContain("Updates are manual");
  } finally {
    act(() => root.unmount());
    vi.unstubAllGlobals();
  }
});
