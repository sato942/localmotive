import { useState } from "react";

export function PathText({ value, label }: { value: string; label: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <span className="path-text">
      <span className="path-text-value" tabIndex={0} title={value} aria-label={`${label}: ${value}`}>
        {value}
      </span>
      <button
        type="button"
        className="path-copy"
        aria-label={`Copy ${label}`}
        onClick={(event) => {
          // The inventory row itself opens the profile on click; copying a
          // path must not hijack the user into another screen (S-24 V1).
          event.stopPropagation();
          void navigator.clipboard
            ?.writeText(value)
            .then(() => {
              setCopied(true);
              window.setTimeout(() => setCopied(false), 1500);
            })
            .catch(() => setCopied(false));
        }}
      >
        {copied ? "Copied" : "Copy"}
      </button>
    </span>
  );
}
