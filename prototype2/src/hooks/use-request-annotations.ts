import * as React from "react";
import type { HighlightColor } from "@/lib/types";

export interface RequestAnnotation {
  highlight: HighlightColor;
  comment: string;
}

const DEFAULT: RequestAnnotation = { highlight: "none", comment: "" };

const store = new Map<string, RequestAnnotation>();
const listeners = new Set<() => void>();

function emit(): void {
  listeners.forEach((l) => l());
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

function update(id: string, patch: Partial<RequestAnnotation>): void {
  store.set(id, { ...(store.get(id) ?? DEFAULT), ...patch });
  emit();
}

export function useRequestAnnotations() {
  const [, force] = React.useReducer((x: number) => x + 1, 0);
  React.useEffect(() => subscribe(force), []);
  return {
    get(id: string): RequestAnnotation {
      return store.get(id) ?? DEFAULT;
    },
    setHighlight(id: string, highlight: HighlightColor): void {
      update(id, { highlight });
    },
    setComment(id: string, comment: string): void {
      update(id, { comment });
    },
  };
}