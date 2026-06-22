import {
  Title,
  Stack,
  Card,
  Text,
  List,
  ThemeIcon,
  Stepper,
  Button,
  Group,
} from "@mantine/core";
import {
  IconQuestionMark,
} from "@tabler/icons-react";
import { useState } from "react";
import { useI18n } from "../i18n";

const isMacOS = typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);
const os = isMacOS ? "macos" : "windows";

function useSteps() {
  const { t } = useI18n();

  return [
    {
      label: t("setupGuide.stepInstallLabel"),
      description: os === "macos"
        ? t("setupGuide.step1DescMac")
        : t("setupGuide.step1DescWin"),
      details: os === "macos"
        ? [
            t("setupGuide.step1Mac1"),
            t("setupGuide.step1Mac2"),
            t("setupGuide.step1Mac3"),
            t("setupGuide.step1Mac4"),
          ]
        : [
            t("setupGuide.step1Win1"),
            t("setupGuide.step1Win2"),
            t("setupGuide.step1Win3"),
            t("setupGuide.step1Win4"),
          ],
    },
    {
      label: t("setupGuide.stepConfigureLabel"),
      description: t("setupGuide.step2Desc"),
      details: [
        t("setupGuide.step2Detail1"),
        t("setupGuide.step2Detail2"),
        t("setupGuide.step2Detail3"),
        t("setupGuide.step2Detail4"),
      ],
    },
    {
      label: t("setupGuide.stepVoiceAppsLabel"),
      description: t("setupGuide.step3Desc"),
      details: [
        t("setupGuide.step3Detail1"),
        t("setupGuide.step3Detail2"),
        t("setupGuide.step3Detail3"),
        t("setupGuide.step3Detail4"),
        t("setupGuide.step3Detail5"),
        t("setupGuide.step3Detail6"),
        t("setupGuide.step3Detail7"),
        t("setupGuide.step3Detail8"),
      ],
    },
    {
      label: t("setupGuide.stepTestLabel"),
      description: t("setupGuide.step4Desc"),
      details: [
        t("setupGuide.step4Detail1"),
        t("setupGuide.step4Detail2"),
        t("setupGuide.step4Detail3"),
        t("setupGuide.step4Detail4"),
        t("setupGuide.step4Detail5"),
      ],
    },
  ];
}

function useFaqItems() {
  const { t } = useI18n();

  return [
    { q: t("setupGuide.faqQ1"), a: t("setupGuide.faqA1") },
    { q: t("setupGuide.faqQ2"), a: t("setupGuide.faqA2") },
    { q: t("setupGuide.faqQ3"), a: t("setupGuide.faqA3") },
    { q: t("setupGuide.faqQ4"), a: t("setupGuide.faqA4") },
    { q: t("setupGuide.faqQ5"), a: t("setupGuide.faqA5") },
    { q: t("setupGuide.faqQ6"), a: t("setupGuide.faqA6") },
  ];
}

export default function SetupGuide() {
  const { t } = useI18n();
  const [active, setActive] = useState(0);
  const steps = useSteps();
  const faqItems = useFaqItems();

  return (
    <Stack gap="md">
      <Title order={4}>{t("setupGuide.title")}</Title>

      <Card padding="md" radius="md" withBorder>
        <Text fw={500} size="sm" mb="md">
          {t("setupGuide.stepByStep")}
        </Text>

        <Stepper
          active={active}
          onStepClick={setActive}
          orientation="vertical"
          color="cyan"
          size="sm"
        >
          {steps.map((step, i) => (
            <Stepper.Step
              key={i}
              label={step.label}
              description={step.description}
            >
              <List spacing="xs" mt="sm" mb="md">
                {step.details.map((d, j) => (
                  <List.Item key={j}>
                    <Text size="sm">{d}</Text>
                  </List.Item>
                ))}
              </List>
              <Group>
                {i > 0 && (
                  <Button
                    variant="subtle"
                    size="xs"
                    onClick={() => setActive(i - 1)}
                  >
                    {t("common.back")}
                  </Button>
                )}
                {i < steps.length - 1 && (
                  <Button
                    variant="light"
                    color="cyan"
                    size="xs"
                    onClick={() => setActive(i + 1)}
                  >
                    {t("common.next")}
                  </Button>
                )}
              </Group>
            </Stepper.Step>
          ))}
        </Stepper>
      </Card>

      <Card padding="md" radius="md" withBorder>
        <Text fw={500} size="sm" mb="md">
          {t("setupGuide.platformSpecific")}: {os === "macos" ? "macOS" : "Windows"}
        </Text>

        <Stack gap="sm">
          {os === "macos" ? (
            <>
              <Card padding="sm" radius="sm" withBorder>
                <Text size="sm" fw={500}>
                  {t("setupGuide.discordSetup")}
                </Text>
                <Text size="xs" c="dimmed" style={{ whiteSpace: "pre-line" }}>
                  {t("setupGuide.discordSetupDescMac")}
                </Text>
              </Card>
              <Card padding="sm" radius="sm" withBorder>
                <Text size="sm" fw={500}>
                  {t("setupGuide.wechatQQLabelMac")}
                </Text>
                <Text size="xs" c="dimmed">
                  {t("setupGuide.wechatQQSetupDescMac")}
                </Text>
              </Card>
            </>
          ) : (
            <>
              <Card padding="sm" radius="sm" withBorder>
                <Text size="sm" fw={500}>
                  {t("setupGuide.discordSetup")}
                </Text>
                <Text size="xs" c="dimmed" style={{ whiteSpace: "pre-line" }}>
                  {t("setupGuide.discordSetupDescWin")}
                </Text>
              </Card>
              <Card padding="sm" radius="sm" withBorder>
                <Text size="sm" fw={500}>
                  {t("setupGuide.wechatQQLabelWin")}
                </Text>
                <Text size="xs" c="dimmed">
                  {t("setupGuide.wechatQQSetupDescWin")}
                </Text>
              </Card>
            </>
          )}

          <Card padding="sm" radius="sm" withBorder>
            <Text size="sm" fw={500}>
              {t("setupGuide.inGameVoice")}
            </Text>
            <Text size="xs" c="dimmed">
              {t("setupGuide.inGameDesc")}
              <br />
              {t("setupGuide.commonGames")}
            </Text>
          </Card>
        </Stack>
      </Card>

      {/* FAQ */}
      <Card padding="md" radius="md" withBorder>
        <Text fw={500} size="sm" mb="md">
          {t("setupGuide.faq")}
        </Text>
        <Stack gap="md">
          {faqItems.map((faq, i) => (
            <div key={i}>
              <Group gap={6} mb={2}>
                <ThemeIcon size="sm" radius="xl" color="cyan" variant="light">
                  <IconQuestionMark size={12} />
                </ThemeIcon>
                <Text size="sm" fw={500}>
                  {faq.q}
                </Text>
              </Group>
              <Text size="xs" c="dimmed" ml={28}>
                {faq.a}
              </Text>
            </div>
          ))}
        </Stack>
      </Card>
    </Stack>
  );
}
