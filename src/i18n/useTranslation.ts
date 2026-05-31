import { useState, useCallback, useMemo } from "react";
import type { Language, TranslationKey } from "../i18n/translations";
import { translations } from "../i18n/translations";

const STORAGE_KEY = "omniclip-language";

function getStoredLanguage(): Language {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === "zh" || stored === "en") return stored;
  const browserLang = navigator.language.toLowerCase();
  return browserLang.startsWith("zh") ? "zh" : "en";
}

export function useTranslation() {
  const [language, setLanguageState] = useState<Language>(getStoredLanguage);

  const setLanguage = useCallback((lang: Language) => {
    setLanguageState(lang);
    localStorage.setItem(STORAGE_KEY, lang);
  }, []);

  const t = useCallback(
    (key: TranslationKey, params?: Record<string, string | number>) => {
      const text = translations[language][key] ?? key;
      if (!params) return text;
      return Object.entries(params).reduce(
        (str, [k, v]) => str.replace(`{${k}}`, String(v)),
        text,
      );
    },
    [language],
  );

  const value = useMemo(() => ({ language, setLanguage, t }), [language, setLanguage, t]);

  return value;
}

export type UseTranslationReturn = ReturnType<typeof useTranslation>;
