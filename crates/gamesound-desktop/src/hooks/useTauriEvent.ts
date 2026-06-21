import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import type { RuntimeEvent } from "../types";

export function useTauriEvent(
  handler: (event: RuntimeEvent) => void,
) {
  useEffect(() => {
    const unlisten = listen<RuntimeEvent>("runtime-event", (e) => {
      handler(e.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [handler]);
}
