import {
  Activity,
  Boxes,
  FolderTree,
  Gauge,
  LayoutDashboard,
  Mail,
  Plus,
  Search,
  Settings,
} from "lucide-react";
import {
  Sidebar,
  SidebarContent,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar";
import { Button } from "@/components/ui/button";
import { useRoute } from "@/app/RouteContext";
import type { RouteId } from "@/app/router";
import { Logo } from "@/components/Logo";

type IconComponent = (props: {
  size?: number;
  className?: string;
  "aria-hidden"?: boolean | string;
}) => JSX.Element;

const NAV_GROUPS: {
  title: string;
  items: { id: RouteId; description: string; Icon: IconComponent }[];
}[] = [
  {
    title: "Workspace",
    items: [
      {
        id: "overview",
        description: "System status and project summary",
        Icon: LayoutDashboard as unknown as IconComponent,
      },
      {
        id: "projects",
        description: "Parked and linked projects",
        Icon: FolderTree as unknown as IconComponent,
      },
      {
        id: "runtimes",
        description: "Installed and external Node versions",
        Icon: Boxes as unknown as IconComponent,
      },
      {
        id: "services",
        description: "Managed and external databases",
        Icon: Gauge as unknown as IconComponent,
      },
    ],
  },
  {
    title: "Observability",
    items: [
      {
        id: "mail",
        description: "Captured mail per project",
        Icon: Mail as unknown as IconComponent,
      },
      {
        id: "inspector",
        description: "Recent HTTP requests",
        Icon: Search as unknown as IconComponent,
      },
      {
        id: "diagnostics",
        description: "DNS, CA, daemon, and port status",
        Icon: Activity as unknown as IconComponent,
      },
    ],
  },
  {
    title: "System",
    items: [
      {
        id: "settings",
        description: "Theme, retention, and external tool discovery",
        Icon: Settings as unknown as IconComponent,
      },
    ],
  },
];

function GroupedNav() {
  const { route, setRoute } = useRoute();
  return (
    <>
      {NAV_GROUPS.map((group, groupIndex) => (
        <SidebarMenu key={group.title}>
          {groupIndex > 0 && (
            <SidebarMenuItem>
              <span className="mt-2 px-2 text-[10px] font-semibold uppercase tracking-[0.16em] text-muted-foreground">
                {group.title}
              </span>
            </SidebarMenuItem>
          )}
          {groupIndex === 0 && (
            <SidebarMenuItem>
              <span className="px-2 pt-1 text-[10px] font-semibold uppercase tracking-[0.16em] text-muted-foreground">
                {group.title}
              </span>
            </SidebarMenuItem>
          )}
          {group.items.map((item) => {
            const isActive = route === item.id;
            return (
              <SidebarMenuItem key={item.id}>
                <SidebarMenuButton
                  tooltip={item.description}
                  isActive={isActive}
                  onClick={() => setRoute(item.id)}
                  className="data-[slot=sidebar-menu-button]:p-2!"
                >
                  <item.Icon />
                  <span className="capitalize">{item.id}</span>
                </SidebarMenuButton>
              </SidebarMenuItem>
            );
          })}
        </SidebarMenu>
      ))}
    </>
  );
}

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  return (
    <Sidebar collapsible="offcanvas" variant="inset" {...props}>
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton
              size="lg"
              className="data-[slot=sidebar-menu-button]:p-2!"
              asChild
            >
              <a href="#" className="flex items-center gap-2.5">
                <Logo size={26} />
                <div className="flex flex-col leading-none">
                  <span className="text-[15px] font-semibold tracking-[-0.014em]">
                    Nerd
                  </span>
                  <span className="mt-0.5 font-mono text-[10px] tracking-tight text-muted-foreground">
                    v0.1.0
                  </span>
                </div>
              </a>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>

      <SidebarContent>
        <SidebarMenu>
          <SidebarMenuItem>
            <Button
              size="sm"
              variant="default"
              className="w-full justify-start gap-2"
            >
              <Plus className="size-4" />
              <span>Quick Create</span>
            </Button>
          </SidebarMenuItem>
        </SidebarMenu>

        <GroupedNav />
      </SidebarContent>
    </Sidebar>
  );
}
