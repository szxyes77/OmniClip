import { createContext, useContext } from "react";
import type { UseTranslationReturn } from "./useTranslation";
import { useTranslation } from "./useTranslation";
import type { Language } from "./translations";

interface I18nContextType extends UseTranslationReturn {}

const I18nContext = createContext<I18nContextType | null>(null);

export function I18nProvider({ children }: { children: React.ReactNode }) {
  const translation = useTranslation();
  return <I18nContext.Provider value={translation}>{children}</I18nContext.Provider>;
}

export function useI18n() {
  const ctx = useContext(I18nContext);
  if (!ctx) throw new Error("useI18n must be used within I18nProvider");
  return ctx;
}

export type { Language };
