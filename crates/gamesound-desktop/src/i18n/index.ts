import { useUiStore } from "../stores/useUiStore";
import en from "./locales/en";
import zh from "./locales/zh";

export type Locale = "en" | "zh";

const messages: Record<Locale, typeof en> = { en, zh };

/* eslint-disable @typescript-eslint/no-explicit-any */
type NestedKeyOf<T> = T extends Record<string, any>
  ? {
      [K in keyof T & string]: T[K] extends Record<string, any>
        ? `${K}.${NestedKeyOf<T[K]>}`
        : K;
    }[keyof T & string]
  : never;
/* eslint-enable @typescript-eslint/no-explicit-any */

export type I18nKey = NestedKeyOf<typeof en>;

/* eslint-disable @typescript-eslint/no-explicit-any */
function getNestedValue(obj: Record<string, any>, path: string): string {
  const keys = path.split(".");
  let current: any = obj;
/* eslint-enable @typescript-eslint/no-explicit-any */
  for (const k of keys) {
    if (current == null || typeof current !== "object") return path;
    current = current[k];
  }
  return typeof current === "string" ? current : path;
}

export function useI18n() {
  const locale = useUiStore((s) => s.locale);
  const setLocale = useUiStore((s) => s.setLocale);

  const t = (key: I18nKey, params?: Record<string, string | number>): string => {
    let val = getNestedValue(messages[locale] ?? messages.en, key);
    if (params) {
      for (const [k, v] of Object.entries(params)) {
        val = val.replace(`{${k}}`, String(v));
      }
    }
    return val;
  };

  return { t, locale, setLocale };
}
