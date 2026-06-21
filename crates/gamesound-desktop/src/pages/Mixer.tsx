import {
  Title,
  Stack,
  Group,
  Slider,
  Switch,
  Card,
  SimpleGrid,
  Text,
  NumberInput,
  RingProgress,
} from "@mantine/core";
import {
  IconMicrophone,
  IconMusic,
  IconHeadphones,
} from "@tabler/icons-react";
import { useEffect, useCallback } from "react";
import { useMixerStore } from "../stores/useMixerStore";
import { useTauriEvent } from "../hooks/useTauriEvent";
import { useI18n } from "../i18n";
import type { RuntimeEvent } from "../types";



export default function Mixer() {
  const { t } = useI18n();
  const mixer = useMixerStore();

  useEffect(() => {
    mixer.fetchSettings();
  }, []);

  const handleEvent = useCallback(
    (event: RuntimeEvent) => {
      if (event.type === "Levels") {
        mixer.updateLevels(event.data.mic, event.data.output, event.data.monitor);
      }
    },
    [mixer],
  );
  useTauriEvent(handleEvent);

  const settings = mixer.settings;

  return (
    <Stack gap="md" h="100%">
      <Title order={4}>{t("mixer.title")}</Title>

      <SimpleGrid cols={{ base: 1, md: 3 }} spacing="sm">
        {/* Mic Channel */}
        <Card padding="md" radius="md" withBorder>
          <Group gap="sm" mb="md">
            <IconMicrophone size={20} color="var(--mantine-color-cyan-6)" />
            <div>
              <Text fw={500} size="sm">{t("mixer.microphone")}</Text>
              <Text size="xs" c="dimmed">{t("mixer.yourVoice")}</Text>
            </div>
          </Group>

          <Stack gap="md">
            <Group justify="center">
              <RingProgress
                size={80}
                thickness={6}
                roundCaps
                sections={[
                  {
                    value: (settings.mic_muted ? 0 : settings.mic_level) * 100,
                    color: settings.mic_muted
                      ? "gray"
                      : settings.mic_level > 0.8
                        ? "red"
                        : "cyan",
                  },
                ]}
                label={
                  <Text size="xs" ta="center">
                    {Math.round(settings.mic_level * 100)}%
                  </Text>
                }
              />
            </Group>

            <div>
              <Text size="xs" mb={4}>
                {t("mixer.volumePercent", { n: Math.round(settings.mic_volume * 100) })}
              </Text>
              <Slider
                value={settings.mic_volume}
                onChange={(v) => mixer.setMicVolume(v)}
                min={0}
                max={1}
                step={0.05}
                color="cyan"
                disabled={settings.mic_muted}
              />
            </div>

            <Switch
              label={t("mixer.muteMic")}
              checked={settings.mic_muted}
              onChange={() => mixer.toggleMicMute()}
              color="red"
            />
          </Stack>
        </Card>

        {/* SFX Channel */}
        <Card padding="md" radius="md" withBorder>
          <Group gap="sm" mb="md">
            <IconMusic size={20} color="var(--mantine-color-green-6)" />
            <div>
              <Text fw={500} size="sm">{t("mixer.soundEffects")}</Text>
              <Text size="xs" c="dimmed">{t("mixer.yourSoundClips")}</Text>
            </div>
          </Group>

          <Stack gap="md">
            <Group justify="center">
              <RingProgress
                size={80}
                thickness={6}
                roundCaps
                sections={[
                  {
                    value: settings.output_level * 100,
                    color: settings.output_level > 0.8 ? "red" : "cyan",
                  },
                ]}
                label={
                  <Text size="xs" ta="center">
                    {Math.round(settings.output_level * 100)}%
                  </Text>
                }
              />
            </Group>

            <div>
              <Text size="xs" mb={4}>
                {t("mixer.volumePercent", { n: Math.round(settings.sfx_volume * 100) })}
              </Text>
              <Slider
                value={settings.sfx_volume}
                onChange={(v) => mixer.setSfxVolume(v)}
                min={0}
                max={1}
                step={0.05}
                color="cyan"
                disabled={settings.sfx_muted}
              />
            </div>

            <Switch
              label={t("mixer.muteSfx")}
              checked={settings.sfx_muted}
              onChange={() => mixer.toggleSfxMute()}
              color="yellow"
            />
          </Stack>
        </Card>

        {/* Monitor Channel */}
        <Card padding="md" radius="md" withBorder>
          <Group gap="sm" mb="md">
            <IconHeadphones size={20} color="var(--mantine-color-yellow-6)" />
            <div>
              <Text fw={500} size="sm">{t("mixer.monitor")}</Text>
              <Text size="xs" c="dimmed">{t("mixer.localMonitoring")}</Text>
            </div>
          </Group>

          <Stack gap="md">
            <Group justify="center">
              <RingProgress
                size={80}
                thickness={6}
                roundCaps
                sections={[
                  {
                    value: settings.monitor_level * 100,
                    color: settings.monitor_level > 0.8 ? "red" : "cyan",
                  },
                ]}
                label={
                  <Text size="xs" ta="center">
                    {Math.round(settings.monitor_level * 100)}%
                  </Text>
                }
              />
            </Group>

            <div>
              <Text size="xs" mb={4}>
                {t("mixer.volumePercent", { n: Math.round(settings.monitor_volume * 100) })}
              </Text>
              <Slider
                value={settings.monitor_volume}
                onChange={(v) => mixer.setMonitorVolume(v)}
                min={0}
                max={1}
                step={0.05}
                color="cyan"
                disabled={settings.monitor_muted}
              />
            </div>

            <Switch
              label={t("mixer.muteMonitor")}
              checked={settings.monitor_muted}
              onChange={() => mixer.toggleMonitor()}
              color="gray"
            />
          </Stack>
        </Card>
      </SimpleGrid>

      {/* Ducking Settings */}
      <Card padding="md" radius="md" withBorder>
        <Text fw={500} size="sm" mb="md">
          {t("mixer.ducking")}
        </Text>
        <Text size="xs" c="dimmed" mb="md">
          {t("mixer.duckingDesc")}
        </Text>

        <SimpleGrid cols={{ base: 1, md: 2 }} spacing="md">
          <Stack gap="md">
            <Switch
              label={t("mixer.enableDucking")}
              checked={settings.ducking_enabled}
              onChange={(e) =>
                mixer.updateSettings({ ducking_enabled: e.target.checked })
              }
              color="cyan"
            />
            <div>
              <Text size="xs" mb={4}>
                {t("mixer.reductionRatio", { n: Math.round(settings.duck_ratio * 100) })}
              </Text>
              <Slider
                value={settings.duck_ratio}
                onChange={(v) => mixer.updateSettings({ duck_ratio: v })}
                min={0}
                max={1}
                step={0.05}
                color="cyan"
                disabled={!settings.ducking_enabled}
                marks={[
                  { value: 0.2, label: "20%" },
                  { value: 0.5, label: "50%" },
                  { value: 0.8, label: "80%" },
                ]}
              />
            </div>
          </Stack>
          <Stack gap="md">
            <NumberInput
              label={t("mixer.attackMs")}
              value={settings.duck_attack_ms}
              onChange={(v) =>
                mixer.updateSettings({ duck_attack_ms: typeof v === "number" ? v : 50 })
              }
              min={1}
              max={500}
              disabled={!settings.ducking_enabled}
            />
            <NumberInput
              label={t("mixer.releaseMs")}
              value={settings.duck_release_ms}
              onChange={(v) =>
                mixer.updateSettings({ duck_release_ms: typeof v === "number" ? v : 300 })
              }
              min={1}
              max={2000}
              disabled={!settings.ducking_enabled}
            />
            <NumberInput
              label={t("mixer.releaseDelayMs")}
              value={settings.duck_release_delay_ms}
              onChange={(v) =>
                mixer.updateSettings({
                  duck_release_delay_ms: typeof v === "number" ? v : 200,
                })
              }
              min={0}
              max={5000}
              disabled={!settings.ducking_enabled}
            />
          </Stack>
        </SimpleGrid>
      </Card>
    </Stack>
  );
}
