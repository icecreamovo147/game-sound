import { Modal, Text, Stack, Group, Badge, Button, Alert } from "@mantine/core";
import { IconKeyboard, IconAlertCircle } from "@tabler/icons-react";
import { useState, useEffect, useCallback } from "react";
import { useHotkeyStore } from "../stores/useHotkeyStore";
import { useI18n } from "../i18n";
import type { SoundInfo } from "../types";

interface HotkeyCaptureModalProps {
  opened: boolean;
  sound: SoundInfo | null;
  onClose: () => void;
  onBind: (soundId: number, hotkey: string) => Promise<void>;
}

export default function HotkeyCaptureModal({
  opened,
  sound,
  onClose,
  onBind,
}: HotkeyCaptureModalProps) {
  const { t } = useI18n();
  const [isListening, setIsListening] = useState(false);
  const [pressedKeys, setPressedKeys] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [isBinding, setIsBinding] = useState(false);

  const capturedShortcut = useHotkeyStore((s) => s.capturedShortcut);
  const setCaptured = useHotkeyStore((s) => s.setCapturedShortcut);
  const startCapture = useHotkeyStore((s) => s.startCapture);

  useEffect(() => {
    if (capturedShortcut && isListening) {
      setIsListening(false);
      setPressedKeys([capturedShortcut]);
    }
  }, [capturedShortcut, isListening]);

  const handleStartCapture = async () => {
    setError(null);
    setPressedKeys([]);
    setIsListening(true);
    setCaptured(null);

    try {
      await startCapture();
    } catch (e) {
      setError(String(e));
      setIsListening(false);
    }
  };

  const handleBind = async () => {
    if (!sound || pressedKeys.length === 0) return;
    const hotkey = pressedKeys[0];

    setIsBinding(true);
    setError(null);

    try {
      await onBind(sound.id, hotkey);
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setIsBinding(false);
    }
  };

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (!isListening) return;
      e.preventDefault();
      e.stopPropagation();

      const parts: string[] = [];
      if (e.ctrlKey) parts.push("ctrl");
      if (e.altKey) parts.push("alt");
      if (e.shiftKey) parts.push("shift");
      if (e.metaKey) parts.push("meta");

      const keyName = e.key.toLowerCase();
      if (!["control", "alt", "shift", "meta"].includes(keyName)) {
        parts.push(keyName === " " ? "space" : keyName);
      }

      if (parts.length > 1) {
        setPressedKeys([parts.join("+")]);
        setIsListening(false);
      }
    },
    [isListening],
  );

  useEffect(() => {
    if (isListening) {
      window.addEventListener("keydown", handleKeyDown, true);
      return () => window.removeEventListener("keydown", handleKeyDown, true);
    }
  }, [isListening, handleKeyDown]);

  useEffect(() => {
    if (!opened) {
      setIsListening(false);
      setPressedKeys([]);
      setError(null);
      setCaptured(null);
    }
  }, [opened]);

  return (
    <Modal
      opened={opened}
      onClose={onClose}
      title={
        <Group gap="sm">
          <IconKeyboard size={20} />
          <Text fw={600}>{t("hotkeys.bindHotkey")}</Text>
        </Group>
      }
      size="sm"
    >
      <Stack gap="md">
        {sound && (
          <Text size="sm" c="dimmed">
            {t("hotkeys.bindingFor")} <strong>{sound.name}</strong>
          </Text>
        )}

        <Group justify="center" py="md">
          {isListening ? (
            <Badge size="xl" variant="filled" color="cyan" style={{ padding: "12px 24px" }}>
              {t("hotkeys.pressKeys")}
            </Badge>
          ) : pressedKeys.length > 0 ? (
            <Badge size="xl" variant="filled" color="green" style={{ padding: "12px 24px" }}>
              {pressedKeys[0]}
            </Badge>
          ) : (
            <Badge size="xl" variant="outline" color="gray" style={{ padding: "12px 24px" }}>
              {t("hotkeys.noKeyCaptured")}
            </Badge>
          )}
        </Group>

        {error && (
          <Alert icon={<IconAlertCircle size={16} />} color="red" variant="light">
            {error}
          </Alert>
        )}

        <Group justify="center">
          {!isListening && pressedKeys.length === 0 && (
            <Button color="cyan" onClick={handleStartCapture}>
              {t("hotkeys.startCapture")}
            </Button>
          )}
          {pressedKeys.length > 0 && (
            <>
              <Button
                color="cyan"
                onClick={handleBind}
                loading={isBinding}
              >
                {t("hotkeys.confirmAndSave")}
              </Button>
              <Button variant="subtle" color="gray" onClick={handleStartCapture}>
                {t("hotkeys.recapture")}
              </Button>
            </>
          )}
        </Group>
      </Stack>
    </Modal>
  );
}
