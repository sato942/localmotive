import { useEffect, useRef, useState } from "react";
import { parseNumericText } from "./model";

/** Text state for a numeric field (bounded UI input).
 *
 * The field shows exactly what was typed — blank stays blank, "-" stays
 * "-" — while only the parsed number flows to the caller (NaN for blank or
 * unparseable text, so the bounds validators report the problem as text and
 * the action stays blocked). An externally committed finite value resyncs
 * the visible text; the operator's own in-progress typing is never
 * overwritten. */
export function useNumericText(
  value: number,
  onCommit: (next: number) => void,
  step: 1 | "any" = 1,
): readonly [string, (next: string) => void] {
  const [text, setText] = useState(Number.isFinite(value) ? String(value) : "");
  // Resync only when the committed value itself changes (an external edit
  // such as adopting a tuned profile), never on the operator's own
  // keystrokes — even if the parent has not committed them yet.
  const prevValue = useRef(value);
  useEffect(() => {
    if (prevValue.current !== value) {
      prevValue.current = value;
      if (Number.isFinite(value) && parseNumericText(text, step) !== value) {
        setText(String(value));
      }
    }
  }, [value, text, step]);
  const onTextChange = (next: string) => {
    setText(next);
    onCommit(parseNumericText(next, step));
  };
  return [text, onTextChange] as const;
}
