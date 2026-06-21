import {
  Title,
  Stack,
  Group,
  Button,
  Text,
  Badge,
  Card,
  Table,
  ActionIcon,
  Switch,
  Alert,
} from "@mantine/core";
import { modals } from "@mantine/modals";
import {
  IconTrash,
  IconRefresh,
  IconPlus,
  IconAlertCircle,
} from "@tabler/icons-react";
import { useEffect, useState, useCallback } from "react";
import { useHotkeyStore } from "../stores/useHotkeyStore";
import { useSoundStore } from "../stores/useSoundStore";
import { useRuntimeStore } from "../stores/useRuntimeStore";
import { useTauriEvent } from "../hooks/useTauriEvent";
import { useI18n } from "../i18n";
import HotkeyCaptureModal from "../components/HotkeyCaptureModal";
import type { SoundInfo, RuntimeEvent } from "../types";

const isMacOS = typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);

export default function HotkeySettings() {
  const hotkeyStore = useHotkeyStore();
  const soundStore = useSoundStore();
  const runtimeStore = useRuntimeStore();
  const { t } = useI18n();

  const [captureModal, setCaptureModal] = useState(false);
  const [captureTarget, setCaptureTarget] = useState<SoundInfo | null>(null);

  useEffect(() => {
    hotkeyStore.fetchBindings();
    soundStore.fetchSounds();
  }, []);

  const handleEvent = useCallback(
    (event: RuntimeEvent) => {
      if (event.type === "HotkeysRegistered") {
        hotkeyStore.fetchBindings();
      }
    },
    [hotkeyStore],
  );
  useTauriEvent(handleEvent);

  const handleBind = async (soundId: number, hotkey: string) => {
    await hotkeyStore.bindHotkey(soundId, hotkey);
    await soundStore.fetchSounds();
    await hotkeyStore.fetchBindings();
  };

  const handleUnbind = (soundId: number, soundName: string) => {
    modals.openConfirmModal({
      title: t("hotkeys.removeBinding"),
      children: (
        <Text size="sm">
          {t("hotkeys.removeBindingConfirm", { name: soundName })}
        </Text>
      ),
      labels: { confirm: t("common.remove"), cancel: t("common.cancel") },
      confirmProps: { color: "red" },
      onConfirm: async () => {
        await hotkeyStore.unbindHotkey(soundId);
        await soundStore.fetchSounds();
      },
    });
  };

  const handleReregister = () => hotkeyStore.reregisterHotkeys();

  const soundsWithoutHotkeys = soundStore.sounds.filter((s) => !s.hotkey);

  return (
    <Stack gap="md">
      <Group justify="space-between">
        <Title order={4}>{t("hotkeys.title")}</Title>
        <Group gap="xs">
          <Button
            variant="light"
            color="gray"
            size="sm"
            leftSection={<IconRefresh size={16} />}
            onClick={handleReregister}
            loading={hotkeyStore.isLoading}
          >
            {t("hotkeys.reregisterAll")}
          </Button>
        </Group>
      </Group>

      {isMacOS && (
        <Alert
          icon={<IconAlertCircle size={16} />}
          color="yellow"
          variant="light"
          title="macOS Users"
        >
          <Text size="sm">
            {t("hotkeys.macOsAlert")}
          </Text>
        </Alert>
      )}

      {/* Enable/Disable */}
      <Card padding="md" radius="md" withBorder>
        <Group justify="space-between">
          <div>
            <Text fw={500} size="sm">{t("hotkeys.globalHotkeys")}</Text>
            <Text size="xs" c="dimmed">
              {t("hotkeys.hotkeysDesc")}
            </Text>
          </div>
          <Switch
            size="lg"
            onLabel={t("hotkeys.on")}
            offLabel={t("hotkeys.off")}
            checked={runtimeStore.status.hotkeys_enabled}
            onChange={(e) => {
              if (e.target.checked) {
                hotkeyStore.enableHotkeys();
                runtimeStore.updateStatus({ hotkeys_enabled: true });
              } else {
                hotkeyStore.disableHotkeys();
                runtimeStore.updateStatus({ hotkeys_enabled: false });
              }
            }}
            color="cyan"
          />
        </Group>
      </Card>

      {/* Current Bindings */}
      <Card padding="md" radius="md" withBorder>
        <Text fw={500} size="sm" mb="md">
          {t("hotkeys.currentBindings", { n: hotkeyStore.bindings.length })}
        </Text>

        {hotkeyStore.bindings.length === 0 ? (
          <Text size="sm" c="dimmed">
            {t("hotkeys.noBindings")}
          </Text>
        ) : (
          <Table highlightOnHover>
            <Table.Thead>
              <Table.Tr>
                <Table.Th>{t("hotkeys.sound")}</Table.Th>
                <Table.Th>{t("hotkeys.hotkey")}</Table.Th>
                <Table.Th style={{ width: 60 }}>{t("hotkeys.action")}</Table.Th>
              </Table.Tr>
            </Table.Thead>
            <Table.Tbody>
              {hotkeyStore.bindings.map((b) => (
                <Table.Tr key={b.sound_id}>
                  <Table.Td>
                    <Text size="sm">{b.sound_name}</Text>
                  </Table.Td>
                  <Table.Td>
                    <Badge size="sm" color="cyan" variant="filled">
                      {b.hotkey}
                    </Badge>
                  </Table.Td>
                  <Table.Td>
                    <ActionIcon
                      color="red"
                      variant="subtle"
                      size="sm"
                      onClick={() => handleUnbind(b.sound_id, b.sound_name)}
                    >
                      <IconTrash size={14} />
                    </ActionIcon>
                  </Table.Td>
                </Table.Tr>
              ))}
            </Table.Tbody>
          </Table>
        )}
      </Card>

      {/* Unbound Sounds */}
      <Card padding="md" radius="md" withBorder>
        <Text fw={500} size="sm" mb="md">
          {t("hotkeys.unboundSounds", { n: soundsWithoutHotkeys.length })}
        </Text>

        {soundsWithoutHotkeys.length === 0 ? (
          <Text size="sm" c="dimmed">
            {t("hotkeys.allSoundsBound")}
          </Text>
        ) : (
          <Table highlightOnHover>
            <Table.Thead>
              <Table.Tr>
                <Table.Th>{t("hotkeys.sound")}</Table.Th>
                <Table.Th style={{ width: 120 }}>{t("hotkeys.action")}</Table.Th>
              </Table.Tr>
            </Table.Thead>
            <Table.Tbody>
              {soundsWithoutHotkeys.map((s) => (
                <Table.Tr key={s.id}>
                  <Table.Td>
                    <Text size="sm">{s.name}</Text>
                  </Table.Td>
                  <Table.Td>
                    <Button
                      size="xs"
                      variant="light"
                      color="cyan"
                      leftSection={<IconPlus size={12} />}
                      onClick={() => {
                        setCaptureTarget(s);
                        setCaptureModal(true);
                      }}
                    >
                      {t("hotkeys.bind")}
                    </Button>
                  </Table.Td>
                </Table.Tr>
              ))}
            </Table.Tbody>
          </Table>
        )}
      </Card>

      {hotkeyStore.error && (
        <Alert icon={<IconAlertCircle size={16} />} color="red" variant="light">
          {hotkeyStore.error}
        </Alert>
      )}

      <HotkeyCaptureModal
        opened={captureModal}
        sound={captureTarget}
        onClose={() => setCaptureModal(false)}
        onBind={handleBind}
      />
    </Stack>
  );
}
