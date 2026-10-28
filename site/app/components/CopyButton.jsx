"use client";

import { useState } from "react";

// Copies `text` to the clipboard and says so for two seconds.
export default function CopyButton({ text, label = "Copy code", className = "" }) {
  const [copied, setCopied] = useState(false);

  async function copy() {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Clipboard blocked (insecure origin or permissions): leave the button as it was.
    }
  }

  return (
    <button type="button" className={`copy ${className}`} onClick={copy} aria-label={label}>
      <span aria-live="polite">{copied ? "Copied" : "Copy"}</span>
    </button>
  );
}
