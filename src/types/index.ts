export interface ClipboardRecord {
  id: string;
  record_type: "text" | "image" | "files";
  content_hash: string;
  content: string;
  thumbnail_path?: string;
  title: string;
  is_starred: boolean;
  copy_count: number;
  source_app?: string;
  created_at: string;
  updated_at: string;
  last_used_at?: string;
  tags?: { id: string; name: string; color: string }[];
}

export interface Tag {
  id: string;
  name: string;
  color: string;
  created_at: string;
  record_count?: number;
}

export interface AppSettings {
  max_history_count: number;
  auto_cleanup_days: number;
  keep_starred: boolean;
  global_shortcut: string;
  ignore_apps: string[];
  theme: "light" | "dark" | "system";
  master_password_hash?: string;
}

export interface SettingsUpdate {
  max_history_count?: number;
  auto_cleanup_days?: number;
  keep_starred?: boolean;
  global_shortcut?: string;
  theme?: string;
  master_password_hash?: string;
  ignore_apps?: string[];
}

export interface ImportResult {
  imported_records: number;
  imported_tags: number;
  imported_links: number;
  skipped_duplicates: number;
}

export type RecordFilter = "all" | "starred" | "text" | "image" | "files";
