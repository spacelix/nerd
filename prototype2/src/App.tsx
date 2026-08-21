import {
  Panel,
  PanelGroup,
  PanelResizeHandle,
  type ImperativePanelHandle,
} from "react-resizable-panels";
import * as React from "react";

import { Desktop } from "@/components/desktop/desktop";
import { Taskbar } from "@/components/desktop/taskbar";
import { AppSidebar, type Route } from "@/components/shell/app-sidebar";
import { StatusBar } from "@/components/shell/status-bar";
import { InspectorRail } from "@/components/shell/inspector-rail";
import { CommandMenu } from "@/components/shell/command-menu";
import type { WorkingSession } from "@/components/shell/working-session-toggle";
import { ProjectsScreen } from "@/components/projects/projects-screen";
import { ProjectDetailPage } from "@/components/projects/project-detail-page";
import { InspectorScreen } from "@/components/inspector/inspector-screen";
import { MailScreen } from "@/components/mail/mail-screen";
import { RuntimesScreen } from "@/components/runtimes/runtimes-screen";
import { RuntimeDetail } from "@/components/runtimes/runtime-detail";
import { ServicesScreen } from "@/components/services/services-screen";
import { ServiceDetail } from "@/components/services/service-detail";
import { DiagnosticsScreen } from "@/components/diagnostics/diagnostics-screen";
import { DiagnosticDetail } from "@/components/diagnostics/diagnostic-detail";
import { OverviewScreen } from "@/components/overview/overview-screen";
import { SettingsScreen } from "@/components/settings/settings-screen";
import { OnboardingWizard } from "@/components/onboarding/onboarding-wizard";
import { RequestInspector } from "@/components/shell/request-inspector";
import { PreflightDialog } from "@/components/actions/preflight-dialog";
import { ActionHost } from "@/components/actions/action-host";
import {
  diagnosticProbes,
  projects,
  requestDetails,
  runtimes,
  services,
} from "@/mocks/data";

function App() {
  const [route, setRoute] = React.useState<Route>("projects");
  const [commandOpen, setCommandOpen] = React.useState(false);
  const [showOnboarding, setShowOnboarding] = React.useState(
    () => localStorage.getItem("nerd-prototype2-onboarded") !== "1",
  );
  const [workingSession, setWorkingSession] =
    React.useState<WorkingSession>("active");
  const [projectDetailId, setProjectDetailId] = React.useState<string | null>(
    null,
  );
  const [selectedRequestId, setSelectedRequestId] = React.useState<
    string | null
  >(null);
  const [selectedRuntimeId, setSelectedRuntimeId] = React.useState<
    string | null
  >(null);
  const [selectedServiceId, setSelectedServiceId] = React.useState<
    string | null
  >(null);
  const [selectedProbeId, setSelectedProbeId] = React.useState<string | null>(
    null,
  );
  const [bodyCapture, setBodyCapture] = React.useState(false);
  const [inspectorProjectFilter, setInspectorProjectFilter] = React.useState<
    string | null
  >(null);
  const [mailProjectFilter, setMailProjectFilter] = React.useState<string | null>(
    null,
  );
  const inspectorRef = React.useRef<ImperativePanelHandle>(null);
  const [inspectorOpen, setInspectorOpen] = React.useState(true);

  React.useEffect(() => {
    const handler = (e: KeyboardEvent): void => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setCommandOpen(true);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  const navigate = React.useCallback((next: Route, projectId?: string) => {
    setRoute(next);
    setProjectDetailId(projectId ?? null);
    setSelectedRequestId(null);
    setSelectedRuntimeId(null);
    setSelectedServiceId(null);
    setSelectedProbeId(null);
    setMailProjectFilter(null);
  }, []);

  const openMailForProject = React.useCallback((projectId: string) => {
    setRoute("mail");
    setMailProjectFilter(projectId);
  }, []);

  const toggleInspector = React.useCallback(() => {
    const panel = inspectorRef.current;
    if (!panel) return;
    if (inspectorOpen) {
      panel.collapse();
    } else {
      panel.expand();
    }
  }, [inspectorOpen]);

  return (
    <>
      {showOnboarding ? (
        <OnboardingWizard
          onComplete={() => {
            localStorage.setItem("nerd-prototype2-onboarded", "1");
            setShowOnboarding(false);
          }}
        />
      ) : null}
      <Desktop>
        <div className="flex h-full w-full">
          <div className="w-[208px] shrink-0">
            <AppSidebar
              active={route}
              onNavigate={navigate}
              onOpenCommand={() => setCommandOpen(true)}
            />
          </div>
          <div className="flex min-w-0 flex-1 flex-col">
            <main className="flex flex-1 overflow-hidden">
              <PanelGroup direction="horizontal" autoSaveId="nerd-v2-layout">
                <Panel defaultSize={70} minSize={50}>
                  <div className="h-full overflow-y-auto">
                    <CenterStage
                      route={route}
                      workingSession={workingSession}
                      onWorkingSessionChange={setWorkingSession}
                      projectDetailId={projectDetailId}
                      onOpenProject={setProjectDetailId}
                      onBackToProjects={() => setProjectDetailId(null)}
                      selectedRequestId={selectedRequestId}
                      onSelectRequest={setSelectedRequestId}
                      bodyCapture={bodyCapture}
                      onBodyCaptureChange={setBodyCapture}
                      inspectorProjectFilter={inspectorProjectFilter}
                      onInspectorProjectFilterChange={setInspectorProjectFilter}
                      mailProjectFilter={mailProjectFilter}
                      onOpenMail={openMailForProject}
                      onClearMailFilter={() => setMailProjectFilter(null)}
                      selectedRuntimeId={selectedRuntimeId}
                      onSelectRuntime={setSelectedRuntimeId}
                      selectedServiceId={selectedServiceId}
                      onSelectService={setSelectedServiceId}
                      selectedProbeId={selectedProbeId}
                      onSelectProbe={setSelectedProbeId}
                      onNavigate={navigate}
                    />
                  </div>
                </Panel>
                <PanelResizeHandle className="w-px bg-border/60 transition-colors hover:bg-primary/40 data-[resize-handle-state=drag]:bg-primary/60" />
                <Panel
                  ref={inspectorRef}
                  defaultSize={30}
                  minSize={20}
                  maxSize={40}
                  collapsible
                  collapsedSize={0}
                  onCollapse={() => setInspectorOpen(false)}
                  onExpand={() => setInspectorOpen(true)}
                >
<InspectorRail>
                      {buildRailContent({
                        route,
                        selectedRequestId,
                        selectedRuntimeId,
                        selectedServiceId,
                        selectedProbeId,
                        bodyCapture,
                      })}
                    </InspectorRail>
                </Panel>
              </PanelGroup>
            </main>
            <StatusBar
              inspectorOpen={inspectorOpen}
              onToggleInspector={toggleInspector}
            />
          </div>
        </div>
      </Desktop>
      <Taskbar onNavigate={navigate} />
      <PreflightDialog />
      <ActionHost />
      <CommandMenu
        open={commandOpen}
        onOpenChange={setCommandOpen}
        onNavigate={navigate}
        workingSession={workingSession}
      />
    </>
  );
}

interface RailSelection {
  route: Route;
  selectedRequestId: string | null;
  selectedRuntimeId: string | null;
  selectedServiceId: string | null;
  selectedProbeId: string | null;
  bodyCapture?: boolean;
}

function buildRailContent(sel: RailSelection): React.ReactNode {
  if (sel.route === "inspector" && sel.selectedRequestId) {
    const req = requestDetails.find((r) => r.id === sel.selectedRequestId);
    if (req) return <RequestInspector request={req} bodyCapture={sel.bodyCapture} />;
  }
  if (sel.route === "runtimes" && sel.selectedRuntimeId) {
    const rt = runtimes.find((r) => r.id === sel.selectedRuntimeId);
    if (rt) return <RuntimeDetail runtime={rt} />;
  }
  if (sel.route === "services" && sel.selectedServiceId) {
    const svc = services.find((s) => s.id === sel.selectedServiceId);
    if (svc) return <ServiceDetail service={svc} />;
  }
  if (sel.route === "diagnostics" && sel.selectedProbeId) {
    const probe = diagnosticProbes.find((p) => p.id === sel.selectedProbeId);
    if (probe) return <DiagnosticDetail probe={probe} />;
  }
  return null;
}

interface CenterStageProps {
  route: Route;
  workingSession: WorkingSession;
  onWorkingSessionChange: (next: WorkingSession) => void;
  projectDetailId: string | null;
  onOpenProject: (id: string) => void;
  onBackToProjects: () => void;
  selectedRequestId: string | null;
  onSelectRequest: (id: string) => void;
  bodyCapture: boolean;
  onBodyCaptureChange: (v: boolean) => void;
  inspectorProjectFilter: string | null;
  onInspectorProjectFilterChange: (id: string | null) => void;
  mailProjectFilter: string | null;
  onOpenMail: (id: string) => void;
  onClearMailFilter: () => void;
  selectedRuntimeId: string | null;
  onSelectRuntime: (id: string) => void;
  selectedServiceId: string | null;
  onSelectService: (id: string) => void;
  selectedProbeId: string | null;
  onSelectProbe: (id: string) => void;
  onNavigate: (route: Route) => void;
}

function CenterStage({
  route,
  workingSession,
  onWorkingSessionChange,
  projectDetailId,
  onOpenProject,
  onBackToProjects,
  selectedRequestId,
  onSelectRequest,
  bodyCapture,
  onBodyCaptureChange,
  inspectorProjectFilter,
  onInspectorProjectFilterChange,
  mailProjectFilter,
  onOpenMail,
  onClearMailFilter,
  selectedRuntimeId,
  onSelectRuntime,
  selectedServiceId,
  onSelectService,
  selectedProbeId,
  onSelectProbe,
  onNavigate,
}: CenterStageProps) {
  if (route === "projects") {
    if (projectDetailId) {
      const project = projects.find((p) => p.id === projectDetailId);
      if (project) {
        return (
          <ProjectDetailPage
            project={project}
            onBack={onBackToProjects}
            onDeleted={onBackToProjects}
            onOpenMail={onOpenMail}
          />
        );
      }
    }
    return (
      <ProjectsScreen
        workingSession={workingSession}
        onWorkingSessionChange={onWorkingSessionChange}
        onOpenProject={onOpenProject}
      />
    );
  }

  if (route === "inspector") {
    return (
      <InspectorScreen
        selectedId={selectedRequestId}
        onSelect={onSelectRequest}
        bodyCapture={bodyCapture}
        onBodyCaptureChange={onBodyCaptureChange}
        projectFilter={inspectorProjectFilter}
        onProjectFilterChange={onInspectorProjectFilterChange}
      />
    );
  }

  if (route === "mail") {
    return (
      <MailScreen
        projectFilter={mailProjectFilter}
        onClearFilter={onClearMailFilter}
      />
    );
  }

  if (route === "runtimes") {
    return (
      <RuntimesScreen selectedId={selectedRuntimeId} onSelect={onSelectRuntime} />
    );
  }

  if (route === "services") {
    return (
      <ServicesScreen selectedId={selectedServiceId} onSelect={onSelectService} />
    );
  }

  if (route === "diagnostics") {
    return (
      <DiagnosticsScreen
        selectedId={selectedProbeId}
        onSelect={onSelectProbe}
      />
    );
  }

  if (route === "overview") {
    return (
      <OverviewScreen
        workingSession={workingSession}
        onWorkingSessionChange={onWorkingSessionChange}
        onOpenProject={onOpenProject}
        onViewAllProjects={() => onNavigate("projects")}
      />
    );
  }

  if (route === "settings") {
    return <SettingsScreen />;
  }
}

export default App;
