import {
  Paper,
  Title,
  SimpleGrid,
  Group,
  Stack,
  Text,
  Button,
  Badge,
  Card,
  RingProgress,
} from "@mantine/core";
import {
  IconPlayerPlay,
  IconPlayerStop,
  IconRefresh,
  IconAlertCircle,
} from "@tabler/icons-react";
import { useEffect, useCallback } from "react";
import { useRuntimeStore } from "../stores/useRuntimeStore";
import { useDeviceStore } from "../stores/useDeviceStore";
import { useMixerStore } from "../stores/useMixerStore";
import { useSoundStore } from "../stores/useSoundStore";
import { useTauriEvent } from "../hooks/useTauriEvent";
import { useI18n } from "../i18n";
import type { RuntimeEvent } from "../types";

export default function Dashboard() {
  const { t } = useI18n();
  const runtime = useRuntimeStore();
  const devices = useDeviceStore();
  const mixer = useMixerStore();
  const sounds = useSoundStore();

  useEffect(() => {
    runtime.fetchStatus();
    devices.fetchDevices();
    mixer.fetchSettings();
    sounds.fetchSounds();
  }, []);

  const handleEvent = useCallback(
    (event: RuntimeEvent) => {
      switch (event.type) {
        case "Status":
          runtime.updateStatus({ status: event.data.status as "Stopped" | "Running" | "Warning" });
          break;
        case "Levels":
          mixer.updateLevels(event.data.mic, event.data.output, event.data.monitor);
          break;
        case "Error":
          runtime.setError(event.data.message);
          break;
        case "Warning":
          break;
        case "SoundStarted":
        case "SoundStopped":
          break;
      }
    },
    [runtime, mixer],
  );

  useTauriEvent(handleEvent);

  const handleStart = () => runtime.startEngine();
  const handleStop = () => runtime.stopEngine();
  const handleRestart = () => runtime.restartEngine();
  const handleStopAll = () => sounds.stopAll();

  const isRunning = runtime.status.status === "Running";
  const isWarning = runtime.status.status === "Warning";

  return (
    <Stack gap="md" h="100%">
      <Group justify="space-between">
        <Title order={4}>{t("dashboard.title")}</Title>
        <Group gap="xs">
          {isRunning ? (
            <>
              <Button
                color="red"
                leftSection={<IconPlayerStop size={16} />}
                onClick={handleStop}
                loading={runtime.isLoading}
                size="sm"
              >
                {t("runtime.stopEngine")}
              </Button>
              <Button
                variant="subtle"
                color="gray"
                leftSection={<IconRefresh size={16} />}
                onClick={handleRestart}
                loading={runtime.isLoading}
                size="sm"
              >
                {t("runtime.restart")}
              </Button>
            </>
          ) : (
            <Button
              color="cyan"
              leftSection={<IconPlayerPlay size={16} />}
              onClick={handleStart}
              loading={runtime.isLoading}
              size="sm"
            >
              {t("runtime.startEngine")}
            </Button>
          )}
        </Group>
      </Group>

      <SimpleGrid cols={{ base: 1, sm: 2, md: 3, lg: 4 }} spacing="sm">
        <Card padding="md" radius="md" withBorder>
          <Text size="xs" c="dimmed" mb={4}>
            {t("dashboard.engineStatus")}
          </Text>
          <Badge
            size="lg"
            color={isRunning ? "green" : isWarning ? "yellow" : "gray"}
            variant="filled"
          >
            {runtime.status.status}
          </Badge>
        </Card>

        <Card padding="md" radius="md" withBorder>
          <Text size="xs" c="dimmed" mb={4}>
            {t("dashboard.microphone")}
          </Text>
          <Text size="sm" lineClamp={1}>
            {runtime.status.mic_device || t("common.notSet")}
          </Text>
          {!runtime.status.mic_device && isRunning && (
            <Badge size="xs" color="red" variant="light" mt={4}>
              {t("dashboard.noDeviceSelected")}
            </Badge>
          )}
        </Card>

        <Card padding="md" radius="md" withBorder>
          <Text size="xs" c="dimmed" mb={4}>
            {t("dashboard.virtualOutput")}
          </Text>
          <Text size="sm" lineClamp={1}>
            {runtime.status.output_device || t("common.notSet")}
          </Text>
          {!runtime.status.output_device && (
            <Badge size="xs" color="red" variant="light" mt={4}>
              {t("dashboard.requiredConfigureDevices")}
            </Badge>
          )}
        </Card>

        <Card padding="md" radius="md" withBorder>
          <Text size="xs" c="dimmed" mb={4}>
            {t("dashboard.hotkeys")}
          </Text>
          <Badge
            size="lg"
            color={runtime.status.hotkeys_enabled ? "green" : "gray"}
            variant="filled"
          >
            {runtime.status.hotkeys_enabled ? t("common.enabled") : t("common.disabled")}
          </Badge>
        </Card>
      </SimpleGrid>

      <SimpleGrid cols={{ base: 1, lg: 2 }} spacing="sm">
        <Card padding="md" radius="md" withBorder>
          <Text size="sm" fw={500} mb={8}>
            {t("dashboard.audioLevels")}
          </Text>
          <Stack gap="md">
            <Group justify="space-between" wrap="nowrap">
              <Text size="xs" c="dimmed" miw={60}>
                {t("dashboard.mic")}
              </Text>
              <RingProgress
                size={60}
                thickness={4}
                roundCaps
                sections={[
                  {
                    value: mixer.settings.mic_level * 100,
                    color: mixer.settings.mic_level > 0.8 ? "red" : "cyan",
                  },
                ]}
              />
              <Text size="xs" c="dimmed" miw={50} ta="right">
                {Math.round(mixer.settings.mic_level * 100)}%
              </Text>
            </Group>
            <Group justify="space-between" wrap="nowrap">
              <Text size="xs" c="dimmed" miw={60}>
                {t("dashboard.output")}
              </Text>
              <RingProgress
                size={60}
                thickness={4}
                roundCaps
                sections={[
                  {
                    value: mixer.settings.output_level * 100,
                    color: mixer.settings.output_level > 0.8 ? "red" : "cyan",
                  },
                ]}
              />
              <Text size="xs" c="dimmed" miw={50} ta="right">
                {Math.round(mixer.settings.output_level * 100)}%
              </Text>
            </Group>
          </Stack>
        </Card>

        <Card padding="md" radius="md" withBorder>
          <Text size="sm" fw={500} mb={8}>
            {t("dashboard.quickActions")}
          </Text>
          <Stack gap="xs">
            <Button
              variant="light"
              color="red"
              size="sm"
              fullWidth
              onClick={handleStopAll}
            >
              {t("dashboard.stopAllSounds")}
            </Button>
            <Button
              variant="light"
              color="cyan"
              size="sm"
              fullWidth
              onClick={() => sounds.fetchSounds()}
            >
              {t("dashboard.refreshSoundLibrary")}
            </Button>
            <Button
              variant="light"
              color="gray"
              size="sm"
              fullWidth
              onClick={() => devices.fetchDevices()}
            >
              {t("dashboard.refreshAudioDevices")}
            </Button>
          </Stack>
        </Card>
      </SimpleGrid>

      <Card padding="md" radius="md" withBorder>
        <Text size="sm" fw={500} mb={8}>
          {t("dashboard.soundLibraryCount", { n: sounds.sounds.length })}
        </Text>
        {sounds.sounds.length === 0 ? (
          <Text size="sm" c="dimmed">
            {t("dashboard.noSoundsHint")}
          </Text>
        ) : (
          <Group gap="xs">
            {sounds.sounds.slice(0, 10).map((s) => (
              <Badge key={s.id} variant="light" color="gray" size="sm">
                {s.name}
                {s.hotkey && ` [${s.hotkey}]`}
              </Badge>
            ))}
            {sounds.sounds.length > 10 && (
              <Text size="xs" c="dimmed">
                {t("dashboard.moreCount", { n: sounds.sounds.length - 10 })}
              </Text>
            )}
          </Group>
        )}
      </Card>

      {runtime.error && (
        <Card padding="sm" radius="md" style={{ border: "1px solid var(--mantine-color-red-6)" }}>
          <Group gap="xs">
            <IconAlertCircle size={16} color="var(--mantine-color-red-6)" />
            <Text size="sm" c="red" lineClamp={2}>
              {runtime.error}
            </Text>
          </Group>
        </Card>
      )}
    </Stack>
  );
}
