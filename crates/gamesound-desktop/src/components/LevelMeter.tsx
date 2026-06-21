import { Box, Text } from "@mantine/core";

interface LevelMeterProps {
  value: number;
  label: string;
  height?: number;
}

export default function LevelMeter({ value, label, height = 8 }: LevelMeterProps) {
  const clamped = Math.min(1, Math.max(0, value));
  const percentage = clamped * 100;

  const getColor = (v: number) => {
    if (v < 0.5) return "var(--mantine-color-green-6)";
    if (v < 0.8) return "var(--mantine-color-yellow-6)";
    return "var(--mantine-color-red-6)";
  };

  return (
    <Box style={{ width: "100%" }}>
      <Text size="xs" c="dimmed" mb={2}>
        {label}
      </Text>
      <Box
        style={{
          width: "100%",
          height,
          background: "var(--mantine-color-default-hover)",
          borderRadius: 2,
          overflow: "hidden",
        }}
      >
        <Box
          className="level-meter-bar"
          style={{
            width: `${percentage}%`,
            height: "100%",
            background: getColor(clamped),
            borderRadius: 2,
          }}
        />
      </Box>
    </Box>
  );
}
