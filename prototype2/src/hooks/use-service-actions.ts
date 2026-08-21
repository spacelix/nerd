import * as React from "react";

export type ServiceRunStatus = "running" | "stopped" | "degraded" | "starting";

const store = new Map<string, ServiceRunStatus>();
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

export function useServiceActions() {
  const [, force] = React.useReducer((x: number) => x + 1, 0);
  React.useEffect(() => subscribe(force), []);
  return {
    statusFor(s: { id: string; status: string }): ServiceRunStatus {
      return store.get(s.id) ?? (s.status as ServiceRunStatus);
    },
    setStatus(id: string, status: ServiceRunStatus): void {
      store.set(id, status);
      emit();
    },
    start(id: string): void {
      store.set(id, "starting");
      emit();
      window.setTimeout(() => {
        store.set(id, "running");
        emit();
      }, 700);
    },
    stop(id: string): void {
      store.set(id, "stopped");
      emit();
    },
    restart(id: string): void {
      store.set(id, "starting");
      emit();
      window.setTimeout(() => {
        store.set(id, "running");
        emit();
      }, 900);
    },
  };
}