import { invoke } from "@tauri-apps/api/core";
import type { Tag } from "../types";

export async function createTag(name: string, color: string): Promise<Tag> {
  return invoke<Tag>("create_tag", { name, color });
}

export async function getTags(): Promise<Tag[]> {
  return invoke<Tag[]>("get_tags");
}

export async function addTagsToRecord(recordId: string, tagIds: string[]): Promise<void> {
  return invoke<void>("add_tags_to_record", { record_id: recordId, tag_ids: tagIds });
}

export async function renameTag(id: string, newName: string): Promise<Tag> {
  return invoke<Tag>("rename_tag", { id, new_name: newName });
}

export async function deleteTag(id: string): Promise<void> {
  return invoke<void>("delete_tag", { id });
}
