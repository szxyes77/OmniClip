import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ClipboardRecord } from "../types";

export async function getHistory(search = "", limit = 50, offset = 0): Promise<ClipboardRecord[]> {
  return invoke<ClipboardRecord[]>("get_history", { search, limit, offset });
}

export async function getClipboardRecords(limit = 50, offset = 0): Promise<ClipboardRecord[]> {
  return invoke<ClipboardRecord[]>("get_clipboard_records", { limit, offset });
}

export async function searchClips(query: string, limit = 50): Promise<ClipboardRecord[]> {
  return invoke<ClipboardRecord[]>("search_clips", { query, limit });
}

export async function searchRecords(query: string): Promise<ClipboardRecord[]> {
  return invoke<ClipboardRecord[]>("search_records", { query });
}

export async function starItem(id: string): Promise<boolean> {
  return invoke<boolean>("star_item", { id });
}

export async function toggleStarRecord(id: string): Promise<boolean> {
  return invoke<boolean>("toggle_star_record", { id });
}

export async function deleteItem(id: string): Promise<void> {
  return invoke<void>("delete_item", { id });
}

export async function deleteRecord(id: string): Promise<void> {
  return invoke<void>("delete_record", { id });
}

export async function addTags(id: string, tags: string[]): Promise<void> {
  return invoke<void>("add_tags", { id, tags });
}

export async function addTagsToRecord(recordId: string, tagIds: string[]): Promise<void> {
  return invoke<void>("add_tags_to_record", { record_id: recordId, tag_ids: tagIds });
}

export async function pasteRecord(id: string): Promise<void> {
  return invoke<void>("paste_record", { id });
}

export async function showFloatingWindow(): Promise<void> {
  return invoke<void>("show_floating_window");
}

export async function hideFloatingWindow(): Promise<void> {
  return invoke<void>("hide_floating_window");
}

export async function onClipboardUpdate(callback: (record: ClipboardRecord) => void) {
  return listen<ClipboardRecord>("clipboard-update", (event) => {
    callback(event.payload);
  });
}
