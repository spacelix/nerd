import { useQuickActions } from "@/hooks/use-quick-actions";
import {
  AddServiceDialog,
  InstallRuntimeDialog,
  LinkProjectDialog,
  NewProjectDialog,
  ParkDirectoryDialog,
} from "@/components/actions/action-dialogs";

function ActionHost() {
  const quick = useQuickActions();
  const close = (): void => quick.clear();
  const open = quick.pending !== null;

  return (
    <>
      <NewProjectDialog open={open && quick.pending === "new-project"} onOpenChange={close} />
      <ParkDirectoryDialog open={open && quick.pending === "park-directory"} onOpenChange={close} />
      <LinkProjectDialog open={open && quick.pending === "link-project"} onOpenChange={close} />
      <InstallRuntimeDialog open={open && quick.pending === "install-node"} onOpenChange={close} />
      <AddServiceDialog open={open && quick.pending === "add-service"} onOpenChange={close} />
    </>
  );
}

export { ActionHost };