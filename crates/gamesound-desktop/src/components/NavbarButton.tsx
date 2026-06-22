import { UnstyledButton, Tooltip, Text } from "@mantine/core";
import type { IconProps } from "@tabler/icons-react";
import type { FC } from "react";

type IconComponent = FC<IconProps>;

interface NavbarButtonProps {
  icon: IconComponent;
  label: string;
  active: boolean;
  collapsed: boolean;
  onClick: () => void;
}

export default function NavbarButton({
  icon: IconComponent,
  label,
  active,
  collapsed,
  onClick,
}: NavbarButtonProps) {
  return (
    <Tooltip label={label} position="right" disabled={!collapsed} withArrow>
      <UnstyledButton
        onClick={onClick}
        className="navbar-btn"
        data-active={active}
        data-collapsed={collapsed}
        style={{
          width: "100%",
          padding: collapsed ? "10px 0" : "8px 14px",
          display: "flex",
          alignItems: "center",
          gap: 10,
          color: active
            ? "var(--mantine-color-cyan-6)"
            : "var(--mantine-color-dimmed)",
          background: active
            ? "var(--mantine-color-cyan-light)"
            : "transparent",
          borderLeft: active
            ? "3px solid var(--mantine-color-cyan-6)"
            : "3px solid transparent",
          cursor: "pointer",
          transition: "all 0.15s ease",
          justifyContent: collapsed ? "center" : "flex-start",
        }}
      >
        <IconComponent size={20} stroke={1.5} />
        {!collapsed && (
          <Text size="sm" fw={active ? 600 : 400}>
            {label}
          </Text>
        )}
      </UnstyledButton>
    </Tooltip>
  );
}
