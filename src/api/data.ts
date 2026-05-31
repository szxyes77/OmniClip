import { invoke } from "@tauri-apps/api/core";
import type { ImportResult } from "../types";

export async function exportData(path: string): Promise<string> {
  return invoke<string>("export_data", { path });
}

export async function importData(filePath: string): Promise<ImportResult> {
  return invoke<ImportResult>("import_data", { path: filePath });
}
