import { useMemo } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { CheckCircle2, Loader2, RotateCcw, Unplug, XCircle } from "lucide-react";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { vscodePluginsApi } from "@/lib/api";
import type {
  VscodePluginStatus,
  VscodePluginStatusKind,
  VscodePluginTargetId,
} from "@/lib/api";

const queryKey = ["vscode-plugin-targets"] as const;

const statusVariant = (status: VscodePluginStatusKind) => {
  if (status === "managed") return "default";
  if (status === "error") return "destructive";
  if (status === "detected") return "secondary";
  return "outline";
};

const StatusIcon = ({ target }: { target: VscodePluginStatus }) => {
  if (target.status === "managed") {
    return <CheckCircle2 className="h-4 w-4 text-emerald-500" />;
  }
  if (target.status === "error" || !target.installed) {
    return <XCircle className="h-4 w-4 text-muted-foreground" />;
  }
  return <Unplug className="h-4 w-4 text-amber-500" />;
};

const canWritePluginConfig = (target: VscodePluginStatus) =>
  target.installed && target.id !== "codex";

export function VscodePluginsSettings() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();

  const { data, isLoading, error } = useQuery({
    queryKey,
    queryFn: () => vscodePluginsApi.listTargets(),
  });

  const invalidate = async () => {
    await queryClient.invalidateQueries({ queryKey });
  };

  const syncMutation = useMutation({
    mutationFn: (targetId: VscodePluginTargetId) =>
      vscodePluginsApi.syncProvider(targetId),
    onSuccess: async () => {
      toast.success(t("settings.advanced.vscodePlugins.syncSuccess"));
      await invalidate();
    },
    onError: (err) => {
      toast.error(
        err instanceof Error
          ? err.message
          : t("settings.advanced.vscodePlugins.syncFailed"),
      );
    },
  });

  const clearMutation = useMutation({
    mutationFn: (targetId: VscodePluginTargetId) =>
      vscodePluginsApi.clearProvider(targetId),
    onSuccess: async () => {
      toast.success(t("settings.advanced.vscodePlugins.clearSuccess"));
      await invalidate();
    },
    onError: (err) => {
      toast.error(
        err instanceof Error
          ? err.message
          : t("settings.advanced.vscodePlugins.clearFailed"),
      );
    },
  });

  const busyTarget = useMemo(() => {
    if (syncMutation.isPending) return syncMutation.variables;
    if (clearMutation.isPending) return clearMutation.variables;
    return null;
  }, [
    clearMutation.isPending,
    clearMutation.variables,
    syncMutation.isPending,
    syncMutation.variables,
  ]);

  if (isLoading) {
    return (
      <div className="flex items-center gap-2 text-sm text-muted-foreground">
        <Loader2 className="h-4 w-4 animate-spin" />
        {t("settings.advanced.vscodePlugins.loading")}
      </div>
    );
  }

  if (error) {
    return (
      <p className="text-sm text-destructive">
        {t("settings.advanced.vscodePlugins.loadFailed")}
      </p>
    );
  }

  return (
    <div className="grid gap-3 sm:grid-cols-2">
      {(data ?? []).map((target) => {
        const busy = busyTarget === target.id;
        const canWrite = canWritePluginConfig(target);
        return (
          <Card key={target.id} className="rounded-lg">
            <CardContent className="space-y-4 p-4">
              <div className="flex items-start justify-between gap-3">
                <div className="min-w-0 space-y-1">
                  <div className="flex items-center gap-2">
                    <StatusIcon target={target} />
                    <h4 className="truncate text-sm font-semibold">
                      {target.label}
                    </h4>
                  </div>
                  <p className="truncate text-xs text-muted-foreground">
                    {target.extensionId}
                    {target.version ? ` · ${target.version}` : ""}
                  </p>
                </div>
                <Badge variant={statusVariant(target.status)}>
                  {t(`settings.advanced.vscodePlugins.status.${target.status}`)}
                </Badge>
              </div>

              {target.message ? (
                <p className="text-xs text-muted-foreground">
                  {target.message}
                </p>
              ) : null}

              <div className="space-y-1">
                <p className="text-xs font-medium text-muted-foreground">
                  {t("settings.advanced.vscodePlugins.configPaths")}
                </p>
                <div className="space-y-1">
                  {target.configPaths.map((path) => (
                    <code
                      key={path}
                      className="block truncate rounded bg-muted px-2 py-1 text-xs"
                      title={path}
                    >
                      {path}
                    </code>
                  ))}
                </div>
              </div>

              <div className="flex justify-end gap-2">
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  disabled={busy || !canWrite}
                  onClick={() => clearMutation.mutate(target.id)}
                >
                  {busy && clearMutation.variables === target.id ? (
                    <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
                  ) : (
                    <RotateCcw className="mr-2 h-3.5 w-3.5" />
                  )}
                  {t("settings.advanced.vscodePlugins.clear")}
                </Button>
                <Button
                  type="button"
                  size="sm"
                  disabled={busy || !canWrite}
                  onClick={() => syncMutation.mutate(target.id)}
                >
                  {busy && syncMutation.variables === target.id ? (
                    <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
                  ) : (
                    <CheckCircle2 className="mr-2 h-3.5 w-3.5" />
                  )}
                  {t("settings.advanced.vscodePlugins.sync")}
                </Button>
              </div>
            </CardContent>
          </Card>
        );
      })}
    </div>
  );
}
