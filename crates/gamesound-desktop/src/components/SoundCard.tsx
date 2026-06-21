import {
  Card,
  Group,
  Text,
  Badge,
  ActionIcon,
  Slider,
  Menu,
} from "@mantine/core";
import {
  IconPlayerPlay,
  IconPlayerStop,
  IconEdit,
  IconTrash,
  IconDotsVertical,
  IconKeyboard,
} from "@tabler/icons-react";
import { useI18n } from "../i18n";
import type { SoundInfo } from "../types";

interface SoundCardProps {
  sound: SoundInfo;
  isPlaying: boolean;
  onPlay: (id: number) => void;
  onStop: (id: number) => void;
  onEdit: (sound: SoundInfo) => void;
  onDelete: (id: number, name: string) => void;
  onBindHotkey: (sound: SoundInfo) => void;
  onVolumeChange: (id: number, volume: number) => void;
}

export default function SoundCard({
  sound,
  isPlaying,
  onPlay,
  onStop,
  onEdit,
  onDelete,
  onBindHotkey,
  onVolumeChange,
}: SoundCardProps) {
  const { t } = useI18n();
  const fileName = sound.file_path.split("/").pop() || sound.file_path;

  return (
    <Card
      className={isPlaying ? "sound-card-playing" : ""}
      shadow="sm"
      padding="sm"
      radius="md"
      withBorder
      style={{
        transition: "all 0.2s ease",
      }}
    >
      <Group justify="space-between" mb={4}>
        <Text size="sm" fw={500} lineClamp={1} style={{ flex: 1 }}>
          {sound.name}
        </Text>
        <Menu shadow="md" width={160} position="bottom-end">
          <Menu.Target>
            <ActionIcon variant="subtle" color="gray" size="sm">
              <IconDotsVertical size={14} />
            </ActionIcon>
          </Menu.Target>
          <Menu.Dropdown>
            <Menu.Item
              leftSection={<IconEdit size={14} />}
              onClick={() => onEdit(sound)}
            >
              {t("common.edit")}
            </Menu.Item>
            <Menu.Item
              leftSection={<IconKeyboard size={14} />}
              onClick={() => onBindHotkey(sound)}
            >
              {t("soundLibrary.bindHotkey")}
            </Menu.Item>
            <Menu.Divider />
            <Menu.Item
              color="red"
              leftSection={<IconTrash size={14} />}
              onClick={() => onDelete(sound.id, sound.name)}
            >
              {t("common.delete")}
            </Menu.Item>
          </Menu.Dropdown>
        </Menu>
      </Group>

      <Text size="xs" c="dimmed" lineClamp={1} mb={8}>
        {fileName}
      </Text>

      <Group gap={6} mb={8}>
        {sound.hotkey && (
          <Badge
            size="xs"
            variant="light"
            color="cyan"
            leftSection={<IconKeyboard size={10} />}
          >
            {sound.hotkey}
          </Badge>
        )}
        <Badge size="xs" variant="outline" color="gray">
          {sound.playback_mode}
        </Badge>
        {sound.loop_enabled && (
          <Badge size="xs" variant="dot" color="blue">
            {t("soundLibrary.loop")}
          </Badge>
        )}
        {isPlaying && (
          <Badge size="xs" variant="filled" color="green">
            {t("soundLibrary.playing")}
          </Badge>
        )}
      </Group>

      <Text size="xs" c="dimmed" mb={4}>
        {t("mixer.volumePercent", { n: Math.round(sound.volume * 100) })}
      </Text>
      <Slider
        value={sound.volume}
        onChange={(v) => onVolumeChange(sound.id, v)}
        min={0}
        max={1}
        step={0.05}
        size="sm"
        color="cyan"
        styles={{ root: { marginBottom: 8 } }}
      />

      <Group gap={6}>
        {isPlaying ? (
          <ActionIcon
            color="red"
            variant="filled"
            size="md"
            onClick={() => onStop(sound.id)}
          >
            <IconPlayerStop size={16} />
          </ActionIcon>
        ) : (
          <ActionIcon
            color="cyan"
            variant="filled"
            size="md"
            onClick={() => onPlay(sound.id)}
          >
            <IconPlayerPlay size={16} />
          </ActionIcon>
        )}
      </Group>
    </Card>
  );
}
