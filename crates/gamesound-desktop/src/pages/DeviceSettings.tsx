import {
  Title,
  Stack,
  Group,
  Button,
  Text,
  Alert,
  Badge,
  Card,
  SimpleGrid,
  Radio,
} from "@mantine/core";
import {
  IconRefresh,
  IconAlertCircle,
  IconDeviceDesktop,
  IconMicrophone,
  IconHeadphones,
} from "@tabler/icons-react";
import { useEffect } from "react";
import { useDeviceStore } from "../stores/useDeviceStore";
import { useRuntimeStore } from "../stores/useRuntimeStore";
import { useI18n } from "../i18n";
import DeviceSelector from "../components/DeviceSelector";

export default function DeviceSettings() {
  const { t } = useI18n();
  const deviceStore = useDeviceStore();
  const runtimeStore = useRuntimeStore();

  useEffect(() => {
    deviceStore.fetchDevices();
    runtimeStore.fetchStatus();
  }, []);

  const hasVirtualOutput = deviceStore.outputs.some((d) => d.is_virtual);

  return (
    <Stack gap="md" style={{ height: "100%" }}>
      <Group justify="space-between">
        <Title order={4}>{t("devices.title")}</Title>
        <Button
          variant="light"
          color="gray"
          size="sm"
          leftSection={<IconRefresh size={16} />}
          onClick={() => deviceStore.fetchDevices()}
          loading={deviceStore.isLoading}
        >
          {t("devices.refreshDevices")}
        </Button>
      </Group>

      {/* Important notice */}
      <Alert
        icon={<IconAlertCircle size={16} />}
        color="cyan"
        variant="light"
        title={t("devices.howItWorks")}
      >
        <Text size="sm">
          {t("devices.howItWorksDesc")}
        </Text>
      </Alert>

      {!hasVirtualOutput && (
        <Alert
          icon={<IconAlertCircle size={16} />}
          color="yellow"
          variant="light"
          title={t("devices.noVirtualDevice")}
        >
          <Text size="sm">
            {t("devices.installDriver")}
          </Text>
          <Text size="xs" mt={4}>
            <strong>{t("devices.windows")}:</strong> {t("devices.install")}{" "}
            <a
              href="https://vb-audio.com/Cable/"
              target="_blank"
              rel="noreferrer"
            >
              VB-CABLE
            </a>
            <br />
            <strong>{t("devices.macOS")}:</strong> {t("devices.install")}{" "}
            <a
              href="https://existential.audio/blackhole/"
              target="_blank"
              rel="noreferrer"
            >
              BlackHole
            </a>
            {" "}{t("devices.twoChannelVersion")}
          </Text>
        </Alert>
      )}

      <SimpleGrid cols={{ base: 1, md: 2, lg: 3 }} spacing="sm">
        {/* Mic Input */}
        <Card padding="md" radius="md" withBorder>
          <Group gap="sm" mb="md">
            <IconMicrophone size={20} style={{ color: "var(--mantine-color-cyan-6)" }} />
            <div>
              <Text fw={500} size="sm">{t("devices.realMicrophone")}</Text>
              <Text size="xs" c="dimmed">{t("devices.physicalMicDesc")}</Text>
            </div>
          </Group>
          <DeviceSelector
            label={t("devices.inputDevice")}
            devices={deviceStore.inputs}
            value={runtimeStore.status.mic_device}
            onChange={(d) => {
              deviceStore.setMicDevice(d);
              runtimeStore.updateStatus({ mic_device: d });
            }}
            placeholder={t("devices.chooseMicrophone")}
          />
        </Card>

        {/* Virtual Output */}
        <Card padding="md" radius="md" withBorder>
          <Group gap="sm" mb="md">
            <IconDeviceDesktop size={20} style={{ color: "var(--mantine-color-green-6)" }} />
            <div>
              <Text fw={500} size="sm">{t("devices.virtualOutput")}</Text>
              <Text size="xs" c="dimmed">{t("devices.virtualOutputDesc")}</Text>
            </div>
          </Group>
          <DeviceSelector
            label={t("devices.outputDevice")}
            devices={deviceStore.outputs.filter((d) => d.is_virtual)}
            value={runtimeStore.status.output_device}
            onChange={(d) => {
              deviceStore.setOutputDevice(d);
              runtimeStore.updateStatus({ output_device: d });
            }}
            placeholder={t("devices.chooseVirtualDevice")}
          />
          {deviceStore.outputs
            .filter((d) => d.is_virtual)
            .map((d) => (
              <Badge key={d.id} size="xs" color="green" variant="light" mt={4} mr={4}>
                {d.name}
              </Badge>
            ))}
        </Card>

        {/* Monitor */}
        <Card padding="md" radius="md" withBorder>
          <Group gap="sm" mb="md">
            <IconHeadphones size={20} style={{ color: "var(--mantine-color-yellow-6)" }} />
            <div>
              <Text fw={500} size="sm">{t("devices.localMonitor")}</Text>
              <Text size="xs" c="dimmed">{t("devices.localMonitorDesc")}</Text>
            </div>
          </Group>
          <DeviceSelector
            label={t("devices.monitorDevice")}
            devices={deviceStore.outputs}
            value={runtimeStore.status.monitor_device}
            onChange={(d) => {
              deviceStore.setMonitorDevice(d);
              runtimeStore.updateStatus({ monitor_device: d });
            }}
            placeholder={t("devices.chooseMonitorDevice")}
          />
        </Card>
      </SimpleGrid>

      {/* All devices list */}
      <SimpleGrid cols={{ base: 1, md: 2 }} spacing="sm">
        <Card padding="md" radius="md" withBorder>
          <Text fw={500} size="sm" mb="sm">
            {t("devices.inputDevices", { n: deviceStore.inputs.length })}
          </Text>
          <Stack gap={4}>
            {deviceStore.inputs.map((d) => (
              <Group key={d.id} gap="xs" style={{ padding: "4px 0" }}>
                <Radio
                  size="xs"
                  checked={runtimeStore.status.mic_device === d.id}
                  onChange={() => {
                    deviceStore.setMicDevice(d.id);
                    runtimeStore.updateStatus({ mic_device: d.id });
                  }}
                />
                <Text size="sm" style={{ flex: 1 }}>
                  {d.name}
                </Text>
                {d.is_virtual && <Badge size="xs" color="cyan" variant="light">{t("common.virtual")}</Badge>}
              </Group>
            ))}
          </Stack>
        </Card>

        <Card padding="md" radius="md" withBorder>
          <Text fw={500} size="sm" mb="sm">
            {t("devices.outputDevices", { n: deviceStore.outputs.length })}
          </Text>
          <Stack gap={4}>
            {deviceStore.outputs.map((d) => (
              <Group key={d.id} gap="xs" style={{ padding: "4px 0" }}>
                <Radio
                  size="xs"
                  checked={runtimeStore.status.output_device === d.id}
                  onChange={() => {
                    deviceStore.setOutputDevice(d.id);
                    runtimeStore.updateStatus({ output_device: d.id });
                  }}
                />
                <Text size="sm" style={{ flex: 1 }}>
                  {d.name}
                </Text>
                {d.is_virtual && <Badge size="xs" color="cyan" variant="light">{t("common.virtual")}</Badge>}
              </Group>
            ))}
          </Stack>
        </Card>
      </SimpleGrid>

      {deviceStore.error && (
        <Alert icon={<IconAlertCircle size={16} />} color="red" variant="light">
          {deviceStore.error}
        </Alert>
      )}
    </Stack>
  );
}
