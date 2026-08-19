export type ProjectStatus =
  | "running"
  | "starting"
  | "installing"
  | "waiting"
  | "stopped"
  | "degraded"
  | "failed";

export type Framework =
  | "next"
  | "vite-react"
  | "vite-vue"
  | "vite-svelte"
  | "nuxt"
  | "astro"
  | "nestjs"
  | "express"
  | "custom";

export type PackageManager = "npm" | "pnpm" | "yarn";

export type RuntimeOwnership = "managed" | "external" | "degraded";

export type Runtime = {
  id: string;
  version: string;
  channel: "lts" | "current" | "maintenance";
  ownership: RuntimeOwnership;
  default: boolean;
  path: string;
  note?: string;
};

export type ServiceKind = "mysql" | "postgres" | "redis";

export type ServiceStatus = "managed" | "external" | "blocked";

export type Service = {
  id: ServiceKind;
  label: string;
  status: ServiceStatus;
  note?: string;
};

export type ProjectServiceRef = {
  kind: ServiceKind;
  version: string;
  port: number;
  data: string;
  keepRunning: boolean;
};

export type ProjectLogLine = {
  ts: string;
  stream: "stdout" | "stderr" | "system";
  text: string;
};

export type Project = {
  id: string;
  name: string;
  domain: string;
  framework: Framework;
  frameworkLabel: string;
  runtimeId: string;
  packageManager: PackageManager;
  status: ProjectStatus;
  statusDetail?: string;
  port: number;
  autodetected: boolean;
  source: "parked" | "linked";
  path: string;
  trust: "trusted" | "pending";
  logs: ProjectLogLine[];
  services: ProjectServiceRef[];
};
