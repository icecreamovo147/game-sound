import {
  Title,
  Stack,
  Group,
  Button,
  Text,
  Card,
  Switch,
  SegmentedControl,
  Textarea,
  Alert,
  SimpleGrid,
} from "@mantine/core";
import { modals } from "@mantine/modals";
import {
  IconFolder,
  IconFileDownload,
  IconFileUpload,
  IconRestore,
  IconInfoCircle,
  IconAlertCircle,
} from "@tabler/icons-react";
import { useEffect, useState } from "react";
import { useSettingsStore } from "../stores/useSettingsStore";
import { useUiStore } from "../stores/useUiStore";
import { useI18n } from "../i18n";
import type { ColorScheme } from "../stores/useUiStore";
import type { Locale } from "../i18n";

export default function Settings() {
  const { t, locale, setLocale } = useI18n();
  const settingsStore = useSettingsStore();
  const colorScheme = useUiStore((s) => s.colorScheme);
  const setColorScheme = useUiStore((s) => s.setColorScheme);

  const [importText, setImportText] = useState("");
  const [showImport, setShowImport] = useState(false);

  useEffect(() => {
    settingsStore.fetchSettings();
  }, []);

  const settings = settingsStore.settings;

  const handleThemeChange = (v: string) => {
    setColorScheme(v as ColorScheme);
  };

  const handleLanguageChange = (v: string) => {
    const newLocale = v as Locale;
    setLocale(newLocale);
    settingsStore.updateSettings({ language: newLocale });
  };

  const handleResetConfig = () => {
    modals.openConfirmModal({
      title: t("settings.resetConfigTitle"),
      children: (
        <Text size="sm">
          {t("settings.resetConfigDesc")}
        </Text>
      ),
      labels: { confirm: t("common.confirm"), cancel: t("common.cancel") },
      confirmProps: { color: "red" },
      onConfirm: () => settingsStore.resetConfig(),
    });
  };

  return (
    <Stack gap="md">
      <Title order={4}>{t("settings.title")}</Title>

      {/* Theme */}
      <Card padding="md" radius="md" withBorder>
        <Text fw={500} size="sm" mb="md">
          {t("settings.appearance")}
        </Text>
        <SegmentedControl
          value={colorScheme}
          onChange={handleThemeChange}
          data={[
            { label: t("settings.dark"), value: "dark" },
            { label: t("settings.light"), value: "light" },
            { label: t("settings.auto"), value: "auto" },
          ]}
        />
      </Card>

      {/* Language */}
      <Card padding="md" radius="md" withBorder>
        <Text fw={500} size="sm" mb="md">
          {t("settings.language")}
        </Text>
        <SegmentedControl
          value={locale}
          onChange={handleLanguageChange}
          data={[
            { label: t("settings.english"), value: "en" },
            { label: t("settings.chinese"), value: "zh" },
          ]}
        />
      </Card>

      {/* Audio */}
      {settings && (
        <Card padding="md" radius="md" withBorder>
          <Text fw={500} size="sm" mb="md">
            {t("settings.audioSettings")}
          </Text>
          <SimpleGrid cols={{ base: 1, md: 3 }} spacing="sm">
            <div>
              <Text size="xs" c="dimmed" mb={4}>
                {t("settings.sampleRate")}
              </Text>
              <Text size="sm" fw={500}>
                {settings.sample_rate} Hz
              </Text>
            </div>
            <div>
              <Text size="xs" c="dimmed" mb={4}>
                {t("settings.channels")}
              </Text>
              <Text size="sm" fw={500}>
                {settings.channels === 1 ? t("settings.mono") : t("settings.stereo")}
              </Text>
            </div>
            <div>
              <Text size="xs" c="dimmed" mb={4}>
                {t("settings.bufferSize")}
              </Text>
              <Text size="sm" fw={500}>
                {settings.buffer_size} {t("settings.frames")}
              </Text>
            </div>
          </SimpleGrid>
        </Card>
      )}

      {/* Monitor */}
      {settings && (
        <Card padding="md" radius="md" withBorder>
          <Text fw={500} size="sm" mb="md">
            {t("settings.monitorSettings")}
          </Text>
          <Stack gap="md">
            <Switch
              label={t("settings.enableLocalMonitor")}
              checked={settings.monitor_enabled}
              onChange={(e) =>
                settingsStore.updateSettings({
                  monitor_mode: e.target.checked
                    ? settings.monitor_mode === "off"
                      ? "sfx_only"
                      : settings.monitor_mode
                    : "off",
                })
              }
            />
            <SegmentedControl
              value={settings.monitor_mode}
              onChange={(v) =>
                settingsStore.updateSettings({ monitor_mode: v })
              }
              data={[
                { label: t("settings.sfxOnly"), value: "sfx_only" },
                { label: t("settings.fullMix"), value: "full_mix" },
                { label: t("settings.off"), value: "off" },
              ]}
            />
          </Stack>
        </Card>
      )}

      {/* Config & Data */}
      <Card padding="md" radius="md" withBorder>
        <Text fw={500} size="sm" mb="md">
          {t("settings.configAndData")}
        </Text>

        <SimpleGrid cols={{ base: 1, md: 2 }} spacing="sm">
          <Stack gap="xs">
            <Text size="xs" c="dimmed">
              {t("settings.configDirectory")}
            </Text>
            <Button
              variant="light"
              color="gray"
              size="sm"
              leftSection={<IconFolder size={16} />}
              onClick={() => settingsStore.openConfigDir()}
              fullWidth
            >
              {t("settings.openConfigDir")}
            </Button>
            <Button
              variant="light"
              color="gray"
              size="sm"
              leftSection={<IconFileDownload size={16} />}
              onClick={async () => {
                const toml = await settingsStore.exportConfig();
                await navigator.clipboard.writeText(toml);
              }}
              fullWidth
            >
              {t("settings.exportConfig")}
            </Button>
          </Stack>

          <Stack gap="xs">
            <Button
              variant="light"
              color="gray"
              size="sm"
              leftSection={<IconFileUpload size={16} />}
              onClick={() => setShowImport(!showImport)}
              fullWidth
            >
              {t("settings.importConfig")}
            </Button>
            <Button
              variant="light"
              color="red"
              size="sm"
              leftSection={<IconRestore size={16} />}
              onClick={handleResetConfig}
              fullWidth
            >
              {t("settings.resetConfig")}
            </Button>
            <Button
              variant="light"
              color="gray"
              size="sm"
              leftSection={<IconFolder size={16} />}
              onClick={() => settingsStore.openLogDir()}
              fullWidth
            >
              {t("settings.openLogDir")}
            </Button>
          </Stack>
        </SimpleGrid>

        {showImport && (
          <Stack gap="sm" mt="md">
            <Textarea
              label={t("settings.pasteToml")}
              value={importText}
              onChange={(e) => setImportText(e.target.value)}
              minRows={4}
              placeholder="[app]&#10;first_run = false&#10;..."
            />
            <Button
              color="cyan"
              size="sm"
              onClick={async () => {
                await settingsStore.importConfig(importText);
                setImportText("");
                setShowImport(false);
              }}
              disabled={!importText.trim()}
            >
              {t("settings.applyImport")}
            </Button>
          </Stack>
        )}
      </Card>

      {/* Profile */}
      {settingsStore.profile && (
        <Card padding="md" radius="md" withBorder>
          <Group gap="sm">
            <IconInfoCircle size={20} style={{ color: "var(--mantine-color-cyan-6)" }} />
            <div>
              <Text size="sm" fw={500}>
                {t("settings.activeProfile", { name: settingsStore.profile.name })}
              </Text>
              <Text size="xs" c="dimmed">
                {settingsStore.profile.description || t("settings.defaultProfile")}
              </Text>
            </div>
          </Group>
        </Card>
      )}

      {settingsStore.error && (
        <Alert icon={<IconAlertCircle size={16} />} color="red" variant="light">
          {settingsStore.error}
        </Alert>
      )}
    </Stack>
  );
}
