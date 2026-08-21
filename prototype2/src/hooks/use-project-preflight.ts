import * as React from "react";

const listeners = new Set<() => void>();
let pendingId: string | null = null;

function emit(): void {
  listeners.forEach((l) => l());
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function useProjectPreflight() {
  const [, force] = React.useReducer((x: number) => x + 1, 0);
  React.useEffect(() => subscribe(force), []);
  return {
    pendingId,
    request(id: string): void {
      pendingId = id;
      emit();
    },
    cancel(): void {
      pendingId = null;
      emit();
    },
  };
}

export function clearPendingPreflight(): void {
  pendingId = null;
}