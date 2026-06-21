import { Select, Text, Badge } from "@mantine/core";
import { useI18n } from "../i18n";
import type { DeviceInfo } from "../types";

interface DeviceSelectorProps {
  label: string;
  devices: DeviceInfo[];
  value: string | null;
  onChange: (device: string) => void;
  placeholder?: string;
}

export default function DeviceSelector({
  label,
  devices,
  value,
  onChange,
  placeholder,
}: DeviceSelectorProps) {
  const { t } = useI18n();
  const data = devices.map((d) => ({
    value: d.id,
    label: `${d.name}${d.is_virtual ? ` [${t("common.virtual")}]` : ""}`,
  }));

  const virtualCount = devices.filter((d) => d.is_virtual).length;

  return (
    <div>
      <Select
        label={label}
        placeholder={placeholder || t("devices.selectDevice")}
        data={data}
        value={value}
        onChange={(v) => v && onChange(v)}
        searchable
        clearable
      />
      {virtualCount === 0 && (
        <Text size="xs" c="yellow" mt={4}>
          {t("devices.noVirtualDetected")}
        </Text>
      )}
      {value && devices.find((d) => d.id === value)?.is_virtual && (
        <Badge size="xs" color="cyan" variant="light" mt={4}>
          {t("common.virtualDevice")}
        </Badge>
      )}
    </div>
  );
}
