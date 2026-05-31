export type Language = "zh" | "en";

export type TranslationKey =
  | "settings.title"
  | "settings.description"
  | "settings.masterPassword"
  | "settings.masterPasswordDesc"
  | "settings.masterPasswordPlaceholder"
  | "settings.confirmPassword"
  | "settings.dataRetention"
  | "settings.maxHistoryEntries"
  | "settings.autoCleanupDays"
  | "settings.ignoredApps"
  | "settings.ignoredAppsDesc"
  | "settings.ignoreAppsPlaceholder"
  | "settings.dataExportImport"
  | "settings.dataExportImportDesc"
  | "settings.exportData"
  | "settings.importData"
  | "settings.saveSettings"
  | "settings.saving"
  | "settings.saved"
  | "settings.saveFailed"
  | "settings.passwordMismatch"
  | "settings.exportSuccess"
  | "settings.exportFailed"
  | "settings.importSuccess"
  | "settings.importFailed"
  | "settings.language"
  | "settings.chinese"
  | "settings.english"
  | "search.placeholder"
  | "recordList.loading"
  | "recordList.noRecords"
  | "recordList.noRecordsHint"
  | "recordList.copied"
  | "recordList.copyPlain"
  | "recordList.editTags"
  | "recordList.delete"
  | "recordList.tagInputPlaceholder"
  | "tagSidebar.filters"
  | "tagSidebar.allRecords"
  | "tagSidebar.starred"
  | "tagSidebar.tags"
  | "tagSidebar.addTag"
  | "tagSidebar.manageTags"
  | "tagSidebar.noTags"
  | "addTag.title"
  | "addTag.placeholder"
  | "addTag.create"
  | "addTag.creating"
  | "manageTags.title"
  | "manageTags.noTags"
  | "manageTags.noTagsHint"
  | "manageTags.deleteConfirm"
  | "manageTags.yes"
  | "manageTags.no"
  | "app.backToHome"
  | "app.settings";

export const translations: Record<Language, Record<TranslationKey, string>> = {
  zh: {
    "settings.title": "设置",
    "settings.description": "配置你的 OmniClip 偏好选项",
    "settings.masterPassword": "主密码",
    "settings.masterPasswordDesc":
      "设置主密码以派生 AES 加密密钥。该密码将与你的机器指纹结合使用以增强安全性。",
    "settings.masterPasswordPlaceholder": "输入新的主密码（可选）",
    "settings.confirmPassword": "确认密码",
    "settings.dataRetention": "数据保留",
    "settings.maxHistoryEntries": "最大历史条目数",
    "settings.autoCleanupDays": "自动清理天数",
    "settings.ignoredApps": "忽略的应用",
    "settings.ignoredAppsDesc": "来自这些应用的剪贴板内容将不会被保存。",
    "settings.ignoreAppsPlaceholder": "例如：Chrome, 记事本",
    "settings.dataExportImport": "数据导出与导入",
    "settings.dataExportImportDesc":
      "以加密的 JSON 格式导出你的剪贴板历史，或从备份文件导入。",
    "settings.exportData": "导出数据",
    "settings.importData": "导入数据",
    "settings.saveSettings": "保存设置",
    "settings.saving": "保存中...",
    "settings.saved": "设置已保存",
    "settings.saveFailed": "保存失败",
    "settings.passwordMismatch": "密码不匹配",
    "settings.exportSuccess": "数据导出成功",
    "settings.exportFailed": "导出失败",
    "settings.importSuccess":
      "导入成功：{records} 条记录，{tags} 个标签，跳过 {duplicates} 条重复",
    "settings.importFailed": "导入失败，请检查文件格式",
    "settings.language": "语言",
    "settings.chinese": "中文",
    "settings.english": "English",
    "search.placeholder": "搜索剪贴板历史...（Ctrl+Shift+V 打开悬浮窗）",
    "recordList.loading": "加载中...",
    "recordList.noRecords": "未找到剪贴板记录",
    "recordList.noRecordsHint": "开始复制以建立你的历史记录",
    "recordList.copied": "已复制 {count} 次",
    "recordList.copyPlain": "复制纯文本",
    "recordList.editTags": "编辑标签",
    "recordList.delete": "删除",
    "recordList.tagInputPlaceholder": "标签1, 标签2",
    "tagSidebar.filters": "筛选",
    "tagSidebar.allRecords": "全部记录",
    "tagSidebar.starred": "已收藏",
    "tagSidebar.tags": "标签",
    "tagSidebar.addTag": "添加标签",
    "tagSidebar.manageTags": "管理标签",
    "tagSidebar.noTags": "暂无标签",
    "addTag.title": "添加新标签",
    "addTag.placeholder": "输入标签名称...",
    "addTag.create": "创建标签",
    "addTag.creating": "创建中...",
    "manageTags.title": "管理标签",
    "manageTags.noTags": "暂无标签",
    "manageTags.noTagsHint": "从侧边栏创建一个标签。",
    "manageTags.deleteConfirm": "确认删除？",
    "manageTags.yes": "是",
    "manageTags.no": "否",
    "app.backToHome": "← 返回首页",
    "app.settings": "设置",
  },
  en: {
    "settings.title": "Settings",
    "settings.description": "Configure your OmniClip preferences",
    "settings.masterPassword": "Master Password",
    "settings.masterPasswordDesc":
      "Set a master password to derive the AES encryption key. This password will be combined with your machine fingerprint for enhanced security.",
    "settings.masterPasswordPlaceholder": "Enter new master password (optional)",
    "settings.confirmPassword": "Confirm password",
    "settings.dataRetention": "Data Retention",
    "settings.maxHistoryEntries": "Max History Entries",
    "settings.autoCleanupDays": "Auto-Cleanup (Days)",
    "settings.ignoredApps": "Ignored Applications",
    "settings.ignoredAppsDesc":
      "Clipboard content from these applications will not be saved.",
    "settings.ignoreAppsPlaceholder": "e.g., Chrome, Notepad",
    "settings.dataExportImport": "Data Export & Import",
    "settings.dataExportImportDesc":
      "Export your clipboard history in encrypted JSON format, or import from a backup file.",
    "settings.exportData": "Export Data",
    "settings.importData": "Import Data",
    "settings.saveSettings": "Save Settings",
    "settings.saving": "Saving...",
    "settings.saved": "Settings saved",
    "settings.saveFailed": "Save failed",
    "settings.passwordMismatch": "Passwords do not match",
    "settings.exportSuccess": "Data exported successfully",
    "settings.exportFailed": "Export failed",
    "settings.importSuccess":
      "Import successful: {records} records, {tags} tags, skipped {duplicates} duplicates",
    "settings.importFailed": "Import failed, please check the file format",
    "settings.language": "Language",
    "settings.chinese": "中文",
    "settings.english": "English",
    "search.placeholder": "Search clipboard history... (Ctrl+Shift+V for floating window)",
    "recordList.loading": "Loading...",
    "recordList.noRecords": "No clipboard records found",
    "recordList.noRecordsHint": "Start copying to build your history",
    "recordList.copied": "copied {count} times",
    "recordList.copyPlain": "Copy Plain Text",
    "recordList.editTags": "Edit Tags",
    "recordList.delete": "Delete",
    "recordList.tagInputPlaceholder": "tag1, tag2",
    "tagSidebar.filters": "Filters",
    "tagSidebar.allRecords": "All Records",
    "tagSidebar.starred": "Starred",
    "tagSidebar.tags": "Tags",
    "tagSidebar.addTag": "Add Tag",
    "tagSidebar.manageTags": "Manage Tags",
    "tagSidebar.noTags": "No tags yet",
    "addTag.title": "Add New Tag",
    "addTag.placeholder": "Enter tag name...",
    "addTag.create": "Create Tag",
    "addTag.creating": "Creating...",
    "manageTags.title": "Manage Tags",
    "manageTags.noTags": "No tags yet",
    "manageTags.noTagsHint": "Create one from the sidebar.",
    "manageTags.deleteConfirm": "Delete?",
    "manageTags.yes": "Yes",
    "manageTags.no": "No",
    "app.backToHome": "← Back to Home",
    "app.settings": "Settings",
  },
};
