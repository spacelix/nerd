import * as React from "react";

export type ProjectStatus = "running" | "starting" | "stopped";

export type DaemonState =
  | "running"
  | "absent"
  | "protocol-mismatch"
  | "unhealthy";

export interface DaemonSnapshot {
  daemon: {
    connected: boolean;
    version: string;
    ipc: number;
    state: DaemonState;
  };
  project: {
    name: string;
    domain: string;
    status: ProjectStatus;
    port: number;
    runtime: string;
  };
  services: number;
  requests: number;
}

type DaemonEvent =
  | { type: "daemon.heartbeat" }
  | { type: "project.request"; requestId: number }
  | { type: "project.status"; status: ProjectStatus };

const initialSnapshot: DaemonSnapshot = {
  daemon: {
    connected: true,
    version: "v0.1.0-alpha.1",
    ipc: 1,
    state: "running",
  },
  project: {
    name: "app.test",
    domain: "app.test",
    status: "running",
    port: 3000,
    runtime: "node 22.11.0",
  },
  services: 3,
  requests: 0,
};

function reduce(state: DaemonSnapshot, event: DaemonEvent): DaemonSnapshot {
  switch (event.type) {
    case "daemon.heartbeat":
      return {
        ...state,
        daemon: { ...state.daemon, connected: true },
      };
    case "project.request":
      return { ...state, requests: state.requests + 1 };
    case "project.status":
      return { ...state, project: { ...state.project, status: event.status } };
  }
}

type Listener = (event: DaemonEvent) => void;

const listeners = new Set<Listener>();
let timer: number | null = null;
let tick = 0;

function start(): void {
  if (timer !== null) return;
  timer = window.setInterval(() => {
    tick += 1;
    listeners.forEach((listener) => {
      listener({ type: "daemon.heartbeat" });
      listener({ type: "project.request", requestId: tick });
    });
    if (tick % 20 === 0) {
      listeners.forEach((listener) =>
        listener({ type: "project.status", status: "starting" }),
      );
    }
    if (tick % 20 === 2) {
      listeners.forEach((listener) =>
        listener({ type: "project.status", status: "running" }),
      );
    }
  }, 1000);
}

function stop(): void {
  if (timer === null) return;
  window.clearInterval(timer);
  timer = null;
}

function subscribeDaemonStream(listener: Listener): () => void {
  listeners.add(listener);
  start();
  return () => {
    listeners.delete(listener);
    if (listeners.size === 0) stop();
  };
}

export function useDaemonStream(): DaemonSnapshot {
  const [state, setState] = React.useState<DaemonSnapshot>(initialSnapshot);

  React.useEffect(() => {
    return subscribeDaemonStream((event) => {
      setState((prev) => reduce(prev, event));
    });
  }, []);

  return state;
}