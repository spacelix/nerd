export type ProjectStatus =
  | "running"
  | "starting"
  | "stopped"
  | "degraded"
  | "failed"
  | "crashed";

export type RuntimeClass = "managed" | "external" | "degraded";

export type ServiceStatus =
  | { kind: "managed"; healthy: boolean; version: string }
  | { kind: "external"; healthy: boolean; version: string }
  | { kind: "blocked"; reason: string };

export type ProjectRegistryState =
  | "conflict"
  | "invalid-config"
  | "missing-path";

export interface ProjectFailure {
  stage: string;
  cause: string;
  exitCode?: number;
}

export interface Project {
  id: string;
  name: string;
  domain: string;
  framework: string;
  runtime: string;
  port: number;
  status: ProjectStatus;
  pinned: boolean;
  source: "parked" | "linked";
  packageManager: string;
  command: string;
  readiness: string;
  restartPolicy: string;
  autostart: boolean;
  versionSource: "nerd.json" | ".nvmrc" | ".node-version" | "engines.node" | "default";
  overrides?: string[];
  registry?: { kind: ProjectRegistryState; note: string };
  failure?: ProjectFailure;
}

export interface MailAttachment {
  id: string;
  name: string;
  sizeKb: number;
  contentType: string;
}

export interface MailMessage {
  id: string;
  projectId: string;
  from: string;
  to: string;
  subject: string;
  receivedAt: string;
  unread: boolean;
  bodyText: string;
  bodyIsHtml: boolean;
  bodyHtml?: string;
  remoteImages: boolean;
  attachments: MailAttachment[];
  headers: Record<string, string>;
  raw: string;
}

export interface RequestEvent {
  id: string;
  projectId: string;
  method: string;
  url: string;
  status: number;
  durationMs: number;
  startedAt: string;
  contentType: string | null;
}

export type HighlightColor =
  | "none"
  | "red"
  | "yellow"
  | "green"
  | "blue"
  | "purple";

export interface RequestTiming {
  dns: number;
  connect: number;
  ttfb: number;
  download: number;
}

export interface RequestDetail extends RequestEvent {
  protocol: string;
  host: string;
  path: string;
  query: Record<string, string>;
  requestHeaders: Record<string, string>;
  responseHeaders: Record<string, string>;
  requestBody: string;
  responseBody: string;
  originalBytes?: number;
  timing: RequestTiming;
}

export interface Runtime {
  id: string;
  version: string;
  class: RuntimeClass;
  isDefault: boolean;
  usageCount: number;
}

export interface ServiceSummary {
  id: string;
  name: string;
  version: string;
  status: "running" | "stopped" | "degraded";
  port: number;
}

export interface Service {
  id: string;
  name: string;
  version: string;
  engine: string;
  class: "managed" | "external";
  status: "running" | "stopped" | "degraded";
  port: number;
  projectIds: string[];
  blockerId?: string;
  blockerLabel?: string;
}

export type ProbeStatus =
  | "pass"
  | "warn"
  | "fail"
  | "idle"
  | "unsupported-policy"
  | "foreign-conflict";

export interface DiagnosticProbe {
  id: string;
  name: string;
  status: ProbeStatus;
  summary: string;
  detail: string[];
  actionLabel?: string;
  actionSafe?: boolean;
}

export type LogLevel = "info" | "warn" | "error";

export interface LogLine {
  id: string;
  projectId: string;
  time: string;
  level: LogLevel;
  message: string;
}

export type OperationState = "running" | "done" | "failed" | "cancelled";

export interface Operation {
  id: string;
  label: string;
  state: OperationState;
  progress: number;
  startedAt: string;
  cause?: string;
}
