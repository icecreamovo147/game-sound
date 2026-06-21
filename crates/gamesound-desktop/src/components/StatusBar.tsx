import { Paper, Group, Text, Stack, Badge } from "@mantine/core";
import { useMixerStore } from "../stores/useMixerStore";
import { useRuntimeStore } from "../stores/useRuntimeStore";
import { useI18n } from "../i18n";
import LevelMeter from "./LevelMeter";

export default function StatusBar() {
  const { t } = useI18n();
  const mixer = useMixerStore();
  const error = useRuntimeStore((s) => s.error);
  const warning = useRuntimeStore((s) => s.warning);

  const settings = mixer.settings;

  return (
    <Paper
      p="xs"
      style={(t) => ({
        background: "var(--mantine-color-body)",
        borderTop: `1px solid ${t.colors.dark[4]}`,
        height: "100%",
      })}
    >
      <Group justify="space-between" wrap="nowrap" gap="sm" h="100%">
        <Group gap="md" style={{ flex: 1 }}>
          <Stack gap={2} style={{ flex: 1, maxWidth: 200 }}>
            <Group gap={4}>
              <Text size="xs" c="dimmed">
                {t("statusBar.mic")}
              </Text>
              {settings.mic_muted && (
                <Badge size="xs" color="red" variant="filled">
                  {t("statusBar.muted")}
                </Badge>
              )}
            </Group>
            <LevelMeter value={settings.mic_muted ? 0 : settings.mic_level} label="" height={4} />
          </Stack>
          <Stack gap={2} style={{ flex: 1, maxWidth: 200 }}>
            <Group gap={4}>
              <Text size="xs" c="dimmed">
                {t("statusBar.out")}
              </Text>
              {settings.sfx_muted && (
                <Badge size="xs" color="yellow" variant="filled">
                  {t("statusBar.sfxMuted")}
                </Badge>
              )}
            </Group>
            <LevelMeter value={settings.output_level} label="" height={4} />
          </Stack>
          <Stack gap={2} style={{ flex: 1, maxWidth: 200 }}>
            <Text size="xs" c="dimmed">
              {t("statusBar.monitor")}
            </Text>
            <LevelMeter value={settings.monitor_level} label="" height={4} />
          </Stack>
        </Group>
        {(error || warning) && (
          <Text size="xs" c={error ? "red" : "yellow"} lineClamp={1} style={{ maxWidth: 300 }}>
            {error || warning}
          </Text>
        )}
      </Group>
    </Paper>
  );
}
