import {
  Paper,
  Title,
  Group,
  TextInput,
  Select,
  Button,
  SimpleGrid,
  Modal,
  Text,
  Stack,
  Switch,
  Slider,
  SegmentedControl,
  ActionIcon,
  Badge,
} from "@mantine/core";
import { modals } from "@mantine/modals";
import {
  IconSearch,
  IconPlus,
  IconUpload,
  IconCategoryPlus,
  IconPlayerStop,
} from "@tabler/icons-react";
import { useEffect, useState, useCallback, useMemo } from "react";
import { useSoundStore } from "../stores/useSoundStore";
import { useHotkeyStore } from "../stores/useHotkeyStore";
import { useRuntimeStore } from "../stores/useRuntimeStore";
import { useTauriEvent } from "../hooks/useTauriEvent";
import { useI18n } from "../i18n";
import SoundCard from "../components/SoundCard";
import HotkeyCaptureModal from "../components/HotkeyCaptureModal";
import type { SoundInfo, RuntimeEvent, CategoryInfo } from "../types";

export default function SoundLibrary() {
  const { t } = useI18n();
  const soundStore = useSoundStore();
  const hotkeyStore = useHotkeyStore();
  const runtimeStatus = useRuntimeStore((s) => s.status.status);
  const isRunning = runtimeStatus === "Running";

  const [search, setSearch] = useState("");
  const [editModal, setEditModal] = useState<SoundInfo | null>(null);
  const [addModalOpen, setAddModalOpen] = useState(false);
  const [categoryModalOpen, setCategoryModalOpen] = useState(false);
  const [hotkeyModalOpen, setHotkeyModalOpen] = useState(false);
  const [hotkeyTarget, setHotkeyTarget] = useState<SoundInfo | null>(null);
  const [newSoundPath, setNewSoundPath] = useState("");
  const [newSoundName, setNewSoundName] = useState("");
  const [newCategoryName, setNewCategoryName] = useState("");
  const [activeSoundIds, setActiveSoundIds] = useState<Set<number>>(new Set());

  // Edit form state
  const [editName, setEditName] = useState("");
  const [editVolume, setEditVolume] = useState(0.8);
  const [editMode, setEditMode] = useState("overlay");
  const [editLoop, setEditLoop] = useState(false);
  const [editCategory, setEditCategory] = useState<string | null>(null);
  const [editFavorite, setEditFavorite] = useState(false);

  useEffect(() => {
    soundStore.fetchSounds();
    soundStore.fetchCategories();
    hotkeyStore.fetchBindings();
  }, []);

  // Debounced search
  useEffect(() => {
    const timer = setTimeout(() => {
      soundStore.setSearchQuery(search);
      soundStore.fetchSounds();
    }, 300);
    return () => clearTimeout(timer);
  }, [search]);

  const handleEvent = useCallback((event: RuntimeEvent) => {
    if (event.type === "SoundStarted") {
      setActiveSoundIds((prev) => new Set(prev).add(event.data.id));
    } else if (event.type === "SoundStopped") {
      setActiveSoundIds((prev) => {
        const next = new Set(prev);
        next.delete(event.data.id);
        return next;
      });
    }
  }, []);
  useTauriEvent(handleEvent);

  const handlePlay = async (id: number) => {
    await soundStore.playSound(id);
    setActiveSoundIds((prev) => new Set(prev).add(id));
  };

  const handleStop = async (id: number) => {
    await soundStore.stopSound(id);
    setActiveSoundIds((prev) => {
      const next = new Set(prev);
      next.delete(id);
      return next;
    });
  };

  const openEdit = (sound: SoundInfo) => {
    setEditModal(sound);
    setEditName(sound.name);
    setEditVolume(sound.volume);
    setEditMode(sound.playback_mode);
    setEditLoop(sound.loop_enabled);
    setEditCategory(sound.category_id?.toString() ?? null);
    setEditFavorite(sound.favorite);
  };

  const handleSaveEdit = async () => {
    if (!editModal) return;
    await soundStore.updateSound({
      id: editModal.id,
      name: editName,
      volume: editVolume,
      playback_mode: editMode,
      loop_enabled: editLoop,
      category_id: editCategory ? parseInt(editCategory) : null,
      favorite: editFavorite,
    });
    setEditModal(null);
  };

  const handleAddSound = async () => {
    if (!newSoundPath) return;
    await soundStore.addSound(
      newSoundPath,
      newSoundName || null,
      null,
    );
    setAddModalOpen(false);
    setNewSoundPath("");
    setNewSoundName("");
  };

  const handleAddCategory = async () => {
    if (!newCategoryName.trim()) return;
    await soundStore.addCategory(newCategoryName.trim());
    setCategoryModalOpen(false);
    setNewCategoryName("");
  };

  const handleBindHotkey = (sound: SoundInfo) => {
    setHotkeyTarget(sound);
    setHotkeyModalOpen(true);
  };

  const handleHotkeyBind = async (soundId: number, hotkey: string) => {
    await hotkeyStore.bindHotkey(soundId, hotkey);
    await soundStore.fetchSounds();
  };

  const handleVolumeChange = async (id: number, volume: number) => {
    await soundStore.updateSound({ id, volume });
  };

  const handleDelete = (id: number, name: string) => {
    modals.openConfirmModal({
      title: t("soundLibrary.deleteSound"),
      children: (
        <Text size="sm">
          {t("soundLibrary.deleteConfirm", { name })}
        </Text>
      ),
      labels: { confirm: t("common.delete"), cancel: t("common.cancel") },
      confirmProps: { color: "red" },
      onConfirm: () => soundStore.deleteSound(id),
    });
  };

  const categoryOptions = useMemo(
    () => [
      { value: "all", label: t("common.allCategories") },
      ...soundStore.categories.map((c: CategoryInfo) => ({
        value: c.id.toString(),
        label: c.name,
      })),
    ],
    [soundStore.categories, t],
  );

  return (
    <Stack gap="md" h="100%">
      <Group justify="space-between">
        <Title order={4}>{t("soundLibrary.title")}</Title>
        <Group gap="xs">
          <Button
            variant="light"
            color="gray"
            size="sm"
            leftSection={<IconPlus size={14} />}
            onClick={() => setAddModalOpen(true)}
          >
            {t("soundLibrary.addSound")}
          </Button>
          <Button
            variant="light"
            color="gray"
            size="sm"
            leftSection={<IconCategoryPlus size={14} />}
            onClick={() => setCategoryModalOpen(true)}
          >
            {t("soundLibrary.addCategory")}
          </Button>
          <Button
            variant="light"
            color="red"
            size="sm"
            leftSection={<IconPlayerStop size={14} />}
            onClick={() => soundStore.stopAll()}
          >
            {t("soundLibrary.stopAll")}
          </Button>
        </Group>
      </Group>

      {/* Search and filter */}
      <Group gap="xs">
        <TextInput
          placeholder={t("common.search")}
          leftSection={<IconSearch size={14} />}
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          style={{ flex: 1 }}
        />
        <Select
          data={categoryOptions}
          value={soundStore.selectedCategory?.toString() ?? "all"}
          onChange={(v) =>
            soundStore.setSelectedCategory(v && v !== "all" ? parseInt(v) : null)
          }
          clearable
          style={{ width: 180 }}
        />
      </Group>

      {/* Sound grid */}
      {soundStore.sounds.length === 0 ? (
        <Paper p="xl" style={{ textAlign: "center" }}>
          <Text c="dimmed">{t("soundLibrary.noSoundsFound")}</Text>
        </Paper>
      ) : (
        <SimpleGrid cols={{ base: 1, sm: 2, md: 3, lg: 4, xl: 5 }} spacing="sm">
          {soundStore.sounds.map((sound) => (
            <SoundCard
              key={sound.id}
              sound={sound}
              isPlaying={activeSoundIds.has(sound.id)}
              onPlay={handlePlay}
              onStop={handleStop}
              onEdit={openEdit}
              onDelete={handleDelete}
              onBindHotkey={handleBindHotkey}
              onVolumeChange={handleVolumeChange}
            />
          ))}
        </SimpleGrid>
      )}

      {/* Add Sound Modal */}
      <Modal
        opened={addModalOpen}
        onClose={() => setAddModalOpen(false)}
        title={t("soundLibrary.addSound")}
        size="sm"
      >
        <Stack gap="md">
          <TextInput
            label={t("soundLibrary.filePath")}
            placeholder={t("soundLibrary.filePathPlaceholder")}
            value={newSoundPath}
            onChange={(e) => setNewSoundPath(e.target.value)}
          />
          <TextInput
            label={t("soundLibrary.displayNameOptional")}
            placeholder={t("soundLibrary.soundNamePlaceholder")}
            value={newSoundName}
            onChange={(e) => setNewSoundName(e.target.value)}
          />
          <Text size="xs" c="dimmed">
            {t("soundLibrary.supportedFormats")}
          </Text>
          <Button color="cyan" onClick={handleAddSound} disabled={!newSoundPath}>
            {t("soundLibrary.addSound")}
          </Button>
        </Stack>
      </Modal>

      {/* Edit Sound Modal */}
      <Modal
        opened={!!editModal}
        onClose={() => setEditModal(null)}
        title={editModal ? t("soundLibrary.editTitle", { name: editModal.name }) : t("common.edit")}
        size="sm"
      >
        {editModal && (
          <Stack gap="md">
            <TextInput
              label={t("common.name")}
              value={editName}
              onChange={(e) => setEditName(e.target.value)}
            />
            <Stack gap={4}>
              <Text size="sm">{t("common.volume")}: {Math.round(editVolume * 100)}%</Text>
              <Slider
                value={editVolume}
                onChange={setEditVolume}
                min={0}
                max={1}
                step={0.05}
                color="cyan"
              />
            </Stack>
            <SegmentedControl
              value={editMode}
              onChange={setEditMode}
              data={[
                { label: t("soundLibrary.overlay"), value: "overlay" },
                { label: t("soundLibrary.interrupt"), value: "interrupt" },
                { label: t("soundLibrary.queue"), value: "queue" },
                { label: t("soundLibrary.exclusive"), value: "exclusive" },
              ]}
            />
            <div>
              <Text size="sm" mb={4}>{t("soundLibrary.category")}</Text>
              <Select
                data={soundStore.categories.map((c) => ({
                  value: c.id.toString(),
                  label: c.name,
                }))}
                value={editCategory}
                onChange={setEditCategory}
                placeholder={t("common.noCategory")}
                clearable
              />
            </div>
            <Switch
              label={t("soundLibrary.loop")}
              checked={editLoop}
              onChange={(e) => setEditLoop(e.target.checked)}
            />
            <Switch
              label={t("soundLibrary.favorite")}
              checked={editFavorite}
              onChange={(e) => setEditFavorite(e.target.checked)}
            />
            <Group justify="flex-end">
              <Button variant="subtle" onClick={() => setEditModal(null)}>
                {t("common.cancel")}
              </Button>
              <Button color="cyan" onClick={handleSaveEdit}>
                {t("common.save")}
              </Button>
            </Group>
          </Stack>
        )}
      </Modal>

      {/* Add Category Modal */}
      <Modal
        opened={categoryModalOpen}
        onClose={() => setCategoryModalOpen(false)}
        title={t("soundLibrary.addCategoryTitle")}
        size="sm"
      >
        <Stack gap="md">
          <TextInput
            label={t("soundLibrary.categoryName")}
            value={newCategoryName}
            onChange={(e) => setNewCategoryName(e.target.value)}
          />
          <Button color="cyan" onClick={handleAddCategory} disabled={!newCategoryName.trim()}>
            {t("soundLibrary.add")}
          </Button>
        </Stack>
      </Modal>

      {/* Hotkey Bind Modal */}
      <HotkeyCaptureModal
        opened={hotkeyModalOpen}
        sound={hotkeyTarget}
        onClose={() => setHotkeyModalOpen(false)}
        onBind={handleHotkeyBind}
      />
    </Stack>
  );
}
