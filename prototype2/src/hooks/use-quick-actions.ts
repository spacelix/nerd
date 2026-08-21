import * as React from "react";

export type QuickAction =
  | "new-project"
  | "park-directory"
  | "link-project"
  | "install-node"
  | "add-service";

const listeners = new Set<() => void>();
let pending: QuickAction | null = null;

function emit(): void {
  listeners.forEach((l) => l());
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function useQuickActions() {
  const [, force] = React.useReducer((x: number) => x + 1, 0);
  React.useEffect(() => subscribe(force), []);
  return {
    pending,
    request(action: QuickAction): void {
      pending = action;
      emit();
    },
    clear(): void {
      pending = null;
      emit();
    },
  };
}