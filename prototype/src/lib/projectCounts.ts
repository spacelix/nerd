import type { Project, ProjectStatus } from "@/lib/types";
import { PROJECTS } from "@/mocks/projects";

export type ProjectCounts = Record<ProjectStatus, number>;

const ZERO_COUNTS: ProjectCounts = {
  running: 0,
  starting: 0,
  installing: 0,
  waiting: 0,
  stopped: 0,
  degraded: 0,
  failed: 0,
};

export function countProjectsByStatus(
  projects: readonly Project[] = PROJECTS,
): ProjectCounts {
  const counts: ProjectCounts = { ...ZERO_COUNTS };
  for (const project of projects) {
    counts[project.status] += 1;
  }
  return counts;
}

export function totalProjects(projects: readonly Project[] = PROJECTS): number {
  return projects.length;
}

export function totalRunning(projects: readonly Project[] = PROJECTS): number {
  return projects.filter((p) => p.status === "running").length;
}
