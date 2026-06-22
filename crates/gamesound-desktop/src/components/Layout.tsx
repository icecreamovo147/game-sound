import { AppShell, Burger, Group, Text, Title, Indicator } from "@mantine/core";
import { modals } from "@mantine/modals";
import { useNavigate, useLocation } from "react-router-dom";
import {
  IconDashboard,
  IconMusic,
  IconDeviceSpeaker,
  IconEqual,
  IconKeyboard,
  IconSettings,
  IconHelp,
} from "@tabler/icons-react";
import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useUiStore } from "../stores/useUiStore";
import { useRuntimeStore } from "../stores/useRuntimeStore";
import { useI18n } from "../i18n";
import NavbarButton from "./NavbarButton";
import StatusBar from "./StatusBar";

export default function Layout({ children }: { children: React.ReactNode }) {
  const { t } = useI18n();
  const sidebarCollapsed = useUiStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useUiStore((s) => s.toggleSidebar);
  const runtimeStatus = useRuntimeStore((s) => s.status.status);
  const navigate = useNavigate();
  const location = useLocation();

  const statusColor =
    runtimeStatus === "Running"
      ? "var(--mantine-color-green-6)"
      : runtimeStatus === "Warning"
        ? "var(--mantine-color-yellow-6)"
        : "var(--mantine-color-gray-6)";

  const statusLabel: Record<string, string> = {
    Running: t("runtime.running"),
    Stopped: t("runtime.stopped"),
    Warning: t("runtime.warning"),
  };

  // Listen for native close-dialog event (non-Windows Ask behavior)
  useEffect(() => {
    const unlisten = listen("show-close-dialog", () => {
      modals.openConfirmModal({
        title: t("settings.closeDialogTitle"),
        children: <Text size="sm">{t("settings.closeDialogDesc")}</Text>,
        labels: {
          confirm: t("settings.closeDialogQuit"),
          cancel: t("settings.closeDialogCancel"),
        },
        confirmProps: { color: "red" },
        cancelProps: { color: "gray" },
        onConfirm: () => invoke("confirm_close_window", { action: "quit" }),
        onCancel: () => invoke("confirm_close_window", { action: "minimize_to_tray" }),
      });
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const navItems = [
    { label: t("nav.dashboard"), icon: IconDashboard, path: "/" },
    { label: t("nav.soundLibrary"), icon: IconMusic, path: "/sounds" },
    { label: t("nav.devices"), icon: IconDeviceSpeaker, path: "/devices" },
    { label: t("nav.mixer"), icon: IconEqual, path: "/mixer" },
    { label: t("nav.hotkeys"), icon: IconKeyboard, path: "/hotkeys" },
    { label: t("nav.settings"), icon: IconSettings, path: "/settings" },
    { label: t("nav.setupGuide"), icon: IconHelp, path: "/guide" },
  ];

  return (
    <AppShell
      header={{ height: 40 }}
      navbar={{
        width: sidebarCollapsed ? 60 : 220,
        breakpoint: 0,
      }}
      footer={{ height: 36 }}
      padding="sm"
      style={{ height: "100%" }}
    >
      <AppShell.Header
        style={(t) => ({
          background: "var(--mantine-color-body)",
          borderBottom: `1px solid ${t.colors.dark[4]}`,
        })}
      >
        <Group h="100%" px="xs" justify="space-between" wrap="nowrap">
          <Group gap="xs">
            <Burger
              opened={!sidebarCollapsed}
              onClick={toggleSidebar}
              size="sm"
              color="var(--mantine-color-dimmed)"
            />
            <Title order={5} style={{ letterSpacing: 0.5 }}>
              GameSound
            </Title>
          </Group>
          <Group gap="md">
            <Group gap={6}>
              <Indicator
                color={statusColor}
                size={10}
                processing={runtimeStatus === "Running"}
              />
              <Text size="xs" c="dimmed">
                {statusLabel[runtimeStatus] ?? runtimeStatus}
              </Text>
            </Group>
          </Group>
        </Group>
      </AppShell.Header>

      <AppShell.Navbar
        style={(t) => ({
          background: "var(--mantine-color-body)",
          borderRight: `1px solid ${t.colors.dark[4]}`,
        })}
      >
        {navItems.map((item) => (
          <NavbarButton
            key={item.path}
            icon={item.icon}
            label={item.label}
            active={location.pathname === item.path}
            collapsed={sidebarCollapsed}
            onClick={() => navigate(item.path)}
          />
        ))}
      </AppShell.Navbar>

      <AppShell.Footer>
        <StatusBar />
      </AppShell.Footer>

      <AppShell.Main style={{ overflowY: "auto", minHeight: 0, height: "100%" }}>
        {children}
      </AppShell.Main>
    </AppShell>
  );
}
