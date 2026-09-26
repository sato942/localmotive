// @vitest-environment jsdom
//
// Bounded UI input: a numeric field keeps showing exactly what was typed —
// blank stays blank, "-" stays "-", and only the committed number flows to
// the validators. External value changes still resync the visible text.
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { useState } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useNumericText } from "./useNumericText";

vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);

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
  vi.unstubAllGlobals();
});

function Probe({ initial, onCommit }: { initial: number; onCommit: (next: number) => void }) {
  const [value, setValue] = useState(initial);
  const [text, onTextChange] = useNumericText(value, (next) => {
    setValue(next);
    onCommit(next);
  });
  return (
    <>
      <input
        aria-label="probe"
        value={text}
        onChange={(event) => onTextChange(event.target.value)}
      />
      <button aria-label="external" onClick={() => setValue(9)} />
    </>
  );
}

function renderProbe(initial: number, onCommit: (next: number) => void) {
  act(() => {
    root.render(<Probe initial={initial} onCommit={onCommit} />);
  });
  return container.querySelector('input[aria-label="probe"]') as HTMLInputElement;
}

function typeInto(input: HTMLInputElement, text: string) {
  // The App.catalog pattern: React controlled inputs read the native setter.
  act(() => {
    const proto = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!;
    proto.set!.call(input, text);
    input.dispatchEvent(new Event("input", { bubbles: true }));
  });
}

describe("useNumericText (bounded UI input)", () => {
  it("shows the committed value, keeps incomplete typing visible, and resyncs on external change", () => {
    const committed: number[] = [];
    const input = renderProbe(12, (next) => void committed.push(next));
    expect(input.value).toBe("12");

    typeInto(input, "-");
    expect(input.value).toBe("-");
    expect(committed).toEqual([Number.NaN]);

    typeInto(input, "");
    expect(input.value).toBe("");
    expect(committed).toEqual([Number.NaN, Number.NaN]);

    typeInto(input, "7");
    expect(input.value).toBe("7");
    expect(committed[committed.length - 1]).toBe(7);

    act(() => {
      container.querySelector('button[aria-label="external"]')?.dispatchEvent(
        new MouseEvent("click", { bubbles: true }),
      );
    });
    expect(input.value).toBe("9");
  });
});
