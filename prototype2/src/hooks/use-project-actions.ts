import * as React from "react";
import type { ProjectStatus } from "@/lib/types";

const store = new Map<string, ProjectStatus>();
const removed = new Set<string>();
const trusted = new Set<string>();
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

export function useProjectActions() {
  const [, force] = React.useReducer((x: number) => x + 1, 0);
  React.useEffect(() => subscribe(force), []);
  return {
    statusFor(p: { id: string; status: ProjectStatus }): ProjectStatus {
      return store.get(p.id) ?? p.status;
    },
    setStatus(id: string, status: ProjectStatus): void {
      store.set(id, status);
      emit();
    },
    isRemoved(id: string): boolean {
      return removed.has(id);
    },
    remove(id: string): void {
      removed.add(id);
      emit();
    },
    isTrusted(id: string): boolean {
      return trusted.has(id);
    },
    trust(id: string): void {
      trusted.add(id);
      emit();
    },
  };
}