import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface ClipboardRecord {
  id: string;
  record_type: string;
  content_hash: string;
  content: string;
  thumbnail_path: string | null;
  title: string;
  is_starred: boolean;
  copy_count: number;
  source_app: string | null;
  created_at: string;
  updated_at: string;
  last_used_at: string | null;
  tags: Tag[] | null;
}

export interface Tag {
  id: string;
  name: string;
  color: string;
  created_at: string;
}

export async function getClipboardRecords(limit: number, offset: number): Promise<ClipboardRecord[]> {
  return invoke("get_clipboard_records", { limit, offset });
}

export async function searchRecords(query: string, limit: number): Promise<ClipboardRecord[]> {
  return invoke("search_records", { query, limit });
}

export async function toggleStarRecord(id: string): Promise<boolean> {
  return invoke("toggle_star_record", { id });
}

export async function deleteRecord(id: string): Promise<void> {
  return invoke("delete_record", { id });
}

export async function pasteRecord(id: string): Promise<void> {
  return invoke("paste_record", { id });
}

export async function clearAll(): Promise<number> {
  return invoke("clear_all");
}

export async function getTags(): Promise<Tag[]> {
  return invoke("get_tags");
}

export async function createTag(name: string, color: string): Promise<Tag> {
  return invoke("create_tag", { name, color });
}

export async function deleteTag(id: string): Promise<void> {
  return invoke("delete_tag", { id });
}

export async function toggleMonitor(): Promise<boolean> {
  return invoke("toggle_monitor");
}

export async function getSetting(key: string): Promise<string> {
  return invoke("get_setting", { key });
}

export async function setSetting(key: string, value: string): Promise<void> {
  return invoke("set_setting", { key, value });
}

export async function getLocalIp(): Promise<string> {
  return invoke("get_local_ip");
}

export async function startSync(port: number): Promise<boolean> {
  return invoke("start_sync", { port });
}

export async function stopSync(): Promise<boolean> {
  return invoke("stop_sync", {});
}

export async function syncSendToPeer(ip: string, port: number, text: string): Promise<void> {
  return invoke("sync_send_to_peer", { ip, port, text });
}

export async function syncRequestRecords(ip: string, port: number): Promise<string> {
  return invoke("sync_request_records", { ip, port });
}

export async function getPairCode(): Promise<string> {
  return invoke("get_pair_code");
}

export async function verifyPeerPairCode(code: string): Promise<boolean> {
  return invoke("verify_peer_pair_code", { code });
}

export async function listenClipboardUpdate(callback: (record: ClipboardRecord) => void) {
  return listen<ClipboardRecord>("clipboard-update", (event) => {
    callback(event.payload);
  });
}

export async function listenConfirmQuit(callback: () => void) {
  return listen<void>("confirm-quit", () => {
    callback();
  });
}

export async function listenMonitoringStateChange(callback: (state: boolean) => void) {
  return listen<boolean>("monitoring-state-changed", (event) => {
    callback(event.payload);
  });
}
