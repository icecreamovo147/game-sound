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
      description:
        os === "macos"
          ? "Install BlackHole 2ch"
          : "Install VB-CABLE",
      details:
        os === "macos"
          ? [
              'Download BlackHole from existential.audio/blackhole',
              "Install the 2ch version (not 16ch)",
              "Restart your Mac after installation",
              'Open Audio MIDI Setup → Verify "BlackHole 2ch" appears',
            ]
          : [
              'Download VB-CABLE from vb-audio.com/Cable',
              "Run the installer as Administrator",
              "Restart your computer",
              'Open Sound Settings → Verify "CABLE Input" appears',
            ],
    },
    {
      label: t("setupGuide.stepConfigureLabel"),
      description: "Select devices in Device Settings",
      details: [
        'Navigate to Devices page',
        'Select your real microphone as "Input Device"',
        'Select BlackHole / VB-CABLE as "Virtual Output"',
        "Optionally select headphones as Monitor",
      ],
    },
    {
      label: t("setupGuide.stepVoiceAppsLabel"),
      description: "Set virtual device as mic input",
      details: [
        "Open Discord → User Settings → Voice & Video",
        'Set "Input Device" to BlackHole 2ch / CABLE Output',
        "Disable Echo Cancellation & Noise Suppression",
        "Disable Automatic Gain Control",
        "In QQ/WeChat: Go to Audio Settings",
        'Choose the virtual device as microphone',
        "In-game: Find Voice Chat settings",
        'Select virtual device as input source',
      ],
    },
    {
      label: t("setupGuide.stepTestLabel"),
      description: "Verify the audio chain works",
      details: [
        "Start the audio engine",
        "Add a sound effect and play it",
        "Ask a friend if they can hear both your voice and the effect",
        "Adjust volumes as needed",
        "If no sound: check monitor level, verify virtual device in voice app",
      ],
    },
  ];
}

const faqItems = [
  {
    q: "Why can't my friends hear me?",
    a: "Make sure your real microphone is selected in GameSound. Check that the microphone level meter shows activity when you speak. The engine must be running.",
  },
  {
    q: "Why can't friends hear my sound effects?",
    a: "Verify the virtual output device is selected. In the voice app, set the input to BlackHole 2ch / CABLE Output. Sound effects should have non-zero volume.",
  },
  {
    q: "Why do I hear an echo?",
    a: "Use headphones instead of speakers. Set monitor mode to 'SFX Only'. Disable sidetone/monitoring in your voice app.",
  },
  {
    q: "Global hotkeys don't work on macOS",
    a: "Grant Accessibility permission to the GameSound Desktop app in System Preferences → Security & Privacy → Privacy → Accessibility.",
  },
  {
    q: "The audio sounds distorted or crackly",
    a: "Try increasing the buffer size in Settings. Close other audio-heavy applications. Make sure sample rate matches (48kHz recommended).",
  },
  {
    q: "I can't find BlackHole/VB-CABLE after installing",
    a: "Restart your computer. BlackHole may appear after restart. VB-CABLE creates 'CABLE Input' and 'CABLE Output' devices.",
  },
];

export default function SetupGuide() {
  const { t } = useI18n();
  const [active, setActive] = useState(0);
  const steps = useSteps();

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
                <Text size="xs" c="dimmed">
                  User Settings → Voice & Video → Input Device: BlackHole 2ch
                  <br />
                  Disable: Echo Cancellation, Noise Suppression, Advanced Voice
                  Activity, Automatic Gain Control
                </Text>
              </Card>
              <Card padding="sm" radius="sm" withBorder>
                <Text size="sm" fw={500}>
                  {t("setupGuide.wechatQQSetup")} (macOS)
                </Text>
                <Text size="xs" c="dimmed">
                  Settings → Audio → Microphone: BlackHole 2ch
                </Text>
              </Card>
            </>
          ) : (
            <>
              <Card padding="sm" radius="sm" withBorder>
                <Text size="sm" fw={500}>
                  {t("setupGuide.discordSetup")}
                </Text>
                <Text size="xs" c="dimmed">
                  User Settings → Voice & Video → Input Device: CABLE Output
                  <br />
                  Disable: Echo Cancellation, Noise Suppression, Advanced Voice
                  Activity, Automatic Gain Control
                </Text>
              </Card>
              <Card padding="sm" radius="sm" withBorder>
                <Text size="sm" fw={500}>
                  {t("setupGuide.wechatQQSetup")} (Windows)
                </Text>
                <Text size="xs" c="dimmed">
                  Settings → Audio → Microphone: CABLE Output
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
              Common games: Valorant, CS2, League of Legends, Dota 2, Overwatch
              2, Apex Legends
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
