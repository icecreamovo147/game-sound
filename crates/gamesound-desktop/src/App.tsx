import { BrowserRouter, Routes, Route } from "react-router-dom";
import { MantineProvider } from "@mantine/core";
import { ModalsProvider } from "@mantine/modals";
import { Notifications } from "@mantine/notifications";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useCallback, useEffect } from "react";

import Layout from "./components/Layout";
import Dashboard from "./pages/Dashboard";
import SoundLibrary from "./pages/SoundLibrary";
import DeviceSettings from "./pages/DeviceSettings";
import Mixer from "./pages/Mixer";
import HotkeySettings from "./pages/HotkeySettings";
import Settings from "./pages/Settings";
import SetupGuide from "./pages/SetupGuide";

import { useMixerStore } from "./stores/useMixerStore";
import { useRuntimeStore } from "./stores/useRuntimeStore";
import { useUiStore } from "./stores/useUiStore";
import { useTauriEvent } from "./hooks/useTauriEvent";
import { theme } from "./styles/theme";
import "@mantine/core/styles.css";
import "@mantine/notifications/styles.css";
import "./styles/global.css";

import type { RuntimeEvent } from "./types";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      staleTime: 5000,
    },
  },
});

function AppContent() {
  const mixer = useMixerStore();
  const setRuntimeError = useRuntimeStore((s) => s.setError);
  const setRuntimeWarning = useRuntimeStore((s) => s.setWarning);

  const handleEvent = useCallback(
    (event: RuntimeEvent) => {
      switch (event.type) {
        case "Levels":
          mixer.updateLevels(
            event.data.mic,
            event.data.output,
            event.data.monitor,
          );
          break;
        case "Status":
          break;
        case "Error":
          setRuntimeError(event.data.message);
          setRuntimeWarning(null);
          break;
        case "Warning":
          setRuntimeWarning(event.data.message);
          break;
        case "SoundStarted":
        case "SoundStopped":
          break;
      }
    },
    [mixer, setRuntimeError, setRuntimeWarning],
  );

  useTauriEvent(handleEvent);

  return (
    <Layout>
      <Routes>
        <Route path="/" element={<Dashboard />} />
        <Route path="/sounds" element={<SoundLibrary />} />
        <Route path="/devices" element={<DeviceSettings />} />
        <Route path="/mixer" element={<Mixer />} />
        <Route path="/hotkeys" element={<HotkeySettings />} />
        <Route path="/settings" element={<Settings />} />
        <Route path="/guide" element={<SetupGuide />} />
      </Routes>
    </Layout>
  );
}

export default function App() {
  const colorScheme = useUiStore((s) => s.colorScheme);
  const hydrateFromStorage = useUiStore((s) => s.hydrateFromStorage);

  useEffect(() => {
    hydrateFromStorage();
  }, []);

  return (
    <QueryClientProvider client={queryClient}>
      <MantineProvider
        theme={theme}
        forceColorScheme={colorScheme === "auto" ? undefined : colorScheme}
        defaultColorScheme={colorScheme === "auto" ? "auto" : colorScheme}
      >
        <ModalsProvider>
          <Notifications position="bottom-right" />
          <BrowserRouter>
            <AppContent />
          </BrowserRouter>
        </ModalsProvider>
      </MantineProvider>
    </QueryClientProvider>
  );
}
