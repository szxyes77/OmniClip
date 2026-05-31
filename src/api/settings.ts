import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, SettingsUpdate } from "../types";

export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

export async function updateSettings(settings: SettingsUpdate): Promise<void> {
  return invoke<void>("update_settings", { settings });
}
