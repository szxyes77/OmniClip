import { useState, useEffect, useCallback } from "react";
import {
  getClipboardRecords, pasteRecord, deleteRecord, toggleStarRecord,
  searchRecords, listenClipboardUpdate, clearAll, getSetting, setSetting,
  toggleMonitor, listenConfirmQuit, listenMonitoringStateChange,
  getLocalIp, startSync, stopSync, syncSendToPeer, syncRequestRecords,
  getPairCode, verifyPeerPairCode,
  type ClipboardRecord
} from "./api";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

function formatTime(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMin = Math.floor(diffMs / 60000);
  const diffHour = Math.floor(diffMs / 3600000);
  const diffDay = Math.floor(diffMs / 86400000);
  if (diffMin < 1) return "刚刚";
  if (diffMin < 60) return `${diffMin} 分钟前`;
  if (diffHour < 24) return `${diffHour} 小时前`;
  if (diffDay < 7) return `${diffDay} 天前`;
  return date.toLocaleDateString("zh-CN");
}

const zhTexts = {
  title: "OmniClip",
  recordsTab: "记录",
  settingsTab: "设置",
  syncTab: "同步",
  clearAll: "清空",
  searchPlaceholder: "搜索剪贴板记录...",
  search: "搜索",
  clear: "清除",
  noRecords: "暂无记录，复制内容后会自动保存",
  favorite: "收藏",
  paste: "粘贴",
  delete: "删除",
  confirmClear: "确定要清空所有记录吗？",
  quitTitle: "退出 OmniClip",
  quitMessage: "你想最小化到托盘还是完全退出？",
  minimize: "最小化到托盘",
  exit: "完全退出",
  cancel: "取消",
  generalSettings: "通用设置",
  autoStart: "开机自启动",
  autoStartDesc: "系统启动时自动运行 OmniClip",
  monitoring: "剪贴板监控",
  monitoringDesc: "开启后自动保存复制的内容",
  language: "语言",
  languageDesc: "切换界面显示语言",
  chinese: "中文",
  english: "English",
  sync: "局域网同步",
  syncDesc: "通过 mDNS 自动发现局域网中的其他设备",
  syncPort: "同步端口",
  syncPortDesc: "TCP 监听端口",
  peerIp: "对方 IP",
  peerPort: "对方端口",
  sendToPeer: "发送文本到设备",
  requestRecords: "获取对方记录",
  syncText: "要发送的文本",
  syncTextPlaceholder: "输入要发送到对方剪贴板的文本",
  sendSuccess: "发送成功！",
  sendFailed: "发送失败",
  recordsReceived: "已接收 {} 条记录",
  receiveFailed: "获取失败",
  statusRunning: "运行中",
  statusStopped: "未启动",
  uses: "使用",
  times: "次",
};

const enTexts = {
  title: "OmniClip",
  recordsTab: "Records",
  settingsTab: "Settings",
  syncTab: "Sync",
  clearAll: "Clear",
  searchPlaceholder: "Search clipboard records...",
  search: "Search",
  clear: "Clear",
  noRecords: "No records yet. They will be saved automatically when you copy.",
  favorite: "Favorite",
  paste: "Paste",
  delete: "Delete",
  confirmClear: "Are you sure you want to clear all records?",
  quitTitle: "Quit OmniClip",
  quitMessage: "Do you want to minimize to tray or exit completely?",
  minimize: "Minimize to Tray",
  exit: "Exit Completely",
  cancel: "Cancel",
  generalSettings: "General Settings",
  autoStart: "Auto Start",
  autoStartDesc: "Run OmniClip when system starts",
  monitoring: "Clipboard Monitoring",
  monitoringDesc: "Auto-save copied content when enabled",
  language: "Language",
  languageDesc: "Change interface display language",
  chinese: "中文",
  english: "English",
  sync: "LAN Sync",
  syncDesc: "Auto-discover other devices on the LAN via mDNS",
  syncPort: "Sync Port",
  syncPortDesc: "TCP listener port",
  peerIp: "Peer IP",
  peerPort: "Peer Port",
  sendToPeer: "Send Text to Device",
  requestRecords: "Request Records from Peer",
  syncText: "Text to Send",
  syncTextPlaceholder: "Enter text to send to peer clipboard",
  sendSuccess: "Sent successfully!",
  sendFailed: "Failed to send",
  recordsReceived: "Received {} records",
  receiveFailed: "Failed to receive",
  statusRunning: "Running",
  statusStopped: "Stopped",
  uses: "uses",
  times: "times",
};

type Texts = typeof zhTexts;

export default function App() {
  const [records, setRecords] = useState<ClipboardRecord[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [isSearching, setIsSearching] = useState(false);
  const [activeTab, setActiveTab] = useState<"records" | "settings" | "sync">("records");
  const [autoStart, setAutoStart] = useState(false);
  const [language, setLanguage] = useState("zh");
  const [isMonitoring, setIsMonitoring] = useState(true);
  const [isSyncRunning, setIsSyncRunning] = useState(false);
  const [syncPort, setSyncPort] = useState(18910);
  const [localIp, setLocalIp] = useState("");
  const [peerIp, setPeerIp] = useState("");
  const [peerPort, setPeerPort] = useState(18910);
  const [syncText, setSyncText] = useState("");
  const [syncStatus, setSyncStatus] = useState("");
  const [quitDialogOpen, setQuitDialogOpen] = useState(false);
  const [myPairCode, setMyPairCode] = useState("------");
  const [peerPairCode, setPeerPairCode] = useState("");
  const [isPaired, setIsPaired] = useState(false);
  const [pairStatus, setPairStatus] = useState("");

  const t: Texts = language === "zh" ? zhTexts : enTexts;

  const fetchRecords = useCallback(async () => {
    try {
      const data = await getClipboardRecords(50, 0);
      setRecords(data);
    } catch {}
  }, []);

  const loadSettings = useCallback(async () => {
    try {
      const autoStartVal = await getSetting("auto_start");
      setAutoStart(autoStartVal === "true");
    } catch {}
    try {
      const lang = await getSetting("language");
      if (lang) setLanguage(lang);
    } catch {}
    try {
      const syncVal = await getSetting("sync_enabled");
      setIsSyncRunning(syncVal === "true");
    } catch {}
    try {
      const portVal = await getSetting("sync_port");
      const p = parseInt(portVal);
      if (p > 0) setSyncPort(p);
    } catch {}
  }, []);

  const loadLocalIp = useCallback(async () => {
    try {
      const ip = await getLocalIp();
      setLocalIp(ip);
    } catch {}
  }, []);

  const loadMyPairCode = useCallback(async () => {
    try {
      const code = await getPairCode();
      setMyPairCode(code);
    } catch {}
  }, []);

  const handleVerifyPeerPairCode = async () => {
    if (!peerPairCode || peerPairCode.length !== 6) {
      setPairStatus("请输入6位配对码");
      return;
    }
    try {
      const valid = await verifyPeerPairCode(peerPairCode);
      if (valid) {
        setIsPaired(true);
        setPairStatus("配对成功！");
        setPeerPairCode("");
      } else {
        setPairStatus("配对码无效，请重试");
      }
    } catch (e) {
      setPairStatus("验证失败");
    }
    setTimeout(() => setPairStatus(""), 3000);
  };

  useEffect(() => {
    fetchRecords();
    loadSettings();
    loadLocalIp();
    loadMyPairCode();

    const unsubQuit = listenConfirmQuit(() => {
      setQuitDialogOpen(true);
    });

    const unsubMonitor = listenMonitoringStateChange((state) => {
      setIsMonitoring(state);
    });

    const unsub = listenClipboardUpdate((record) => {
      setRecords((prev) => {
        const exists = prev.some((r) => r.id === record.id);
        if (exists) return prev;
        return [record, ...prev.slice(0, 49)];
      });
    });

    const pollInterval = setInterval(() => {
      fetchRecords();
    }, 3000);

    return () => {
      unsub.then((fn) => fn());
      unsubQuit.then((fn) => fn());
      unsubMonitor.then((fn) => fn());
      clearInterval(pollInterval);
    };
  }, []);

  const handleQuitAction = async (action: "minimize" | "exit") => {
    setQuitDialogOpen(false);
    const appWindow = getCurrentWebviewWindow();
    if (action === "exit") {
      await appWindow.close();
    } else {
      await appWindow.hide();
    }
  };

  const handleSearch = async () => {
    if (!searchQuery.trim()) {
      setIsSearching(false);
      fetchRecords();
      return;
    }
    setIsSearching(true);
    const results = await searchRecords(searchQuery, 50);
    setRecords(results);
  };

  const handleClearSearch = () => {
    setSearchQuery("");
    setIsSearching(false);
    fetchRecords();
  };

  const handlePaste = async (id: string) => {
    try {
      await pasteRecord(id);
    } catch (e) {
      console.error("Paste failed:", e);
    }
  };

  const handleStar = async (id: string) => {
    await toggleStarRecord(id);
    fetchRecords();
  };

  const handleDelete = async (id: string) => {
    await deleteRecord(id);
    setRecords((prev) => prev.filter((r) => r.id !== id));
  };

  const handleClearAll = async () => {
    if (window.confirm(t.confirmClear)) {
      await clearAll();
      setRecords([]);
    }
  };

  const handleAutoStartToggle = async () => {
    const newVal = !autoStart;
    await setSetting("auto_start", newVal ? "true" : "false");
    setAutoStart(newVal);
  };

  const handleLanguageChange = async (lang: string) => {
    await setSetting("language", lang);
    setLanguage(lang);
  };

  const handleToggleMonitor = async () => {
    const newState = await toggleMonitor();
    setIsMonitoring(newState);
  };

  const handleToggleSync = async () => {
    const newVal = !isSyncRunning;
    if (newVal) {
      try {
        await startSync(syncPort);
        setIsSyncRunning(true);
        await setSetting("sync_enabled", "true");
      } catch (e) {
        console.error("Failed to start sync:", e);
      }
    } else {
      try {
        await stopSync();
        setIsSyncRunning(false);
        await setSetting("sync_enabled", "false");
      } catch (e) {
        console.error("Failed to stop sync:", e);
      }
    }
  };

  const handleSyncPortChange = async (val: string) => {
    const p = parseInt(val);
    if (p > 0 && p < 65536) {
      setSyncPort(p);
      await setSetting("sync_port", String(p));
    }
  };

  const handleSendToPeer = async () => {
    if (!syncText.trim() || !peerIp) return;
    try {
      setSyncStatus("sending...");
      await syncSendToPeer(peerIp, peerPort, syncText);
      setSyncStatus(t.sendSuccess);
      setSyncText("");
      setTimeout(() => setSyncStatus(""), 3000);
    } catch {
      setSyncStatus(t.sendFailed);
      setTimeout(() => setSyncStatus(""), 3000);
    }
  };

  const handleRequestRecords = async () => {
    if (!peerIp) return;
    try {
      setSyncStatus("requesting...");
      const json = await syncRequestRecords(peerIp, peerPort);
      const peerRecords: ClipboardRecord[] = JSON.parse(json);
      setRecords((prev) => {
        const existingIds = new Set(prev.map((r) => r.id));
        const newRecords = peerRecords.filter((r) => !existingIds.has(r.id));
        return [...newRecords, ...prev];
      });
      setSyncStatus(t.recordsReceived.replace("{}", String(peerRecords.length)));
      setTimeout(() => setSyncStatus(""), 3000);
    } catch {
      setSyncStatus(t.receiveFailed);
      setTimeout(() => setSyncStatus(""), 3000);
    }
  };

  const TabButton = ({ tab, label }: { tab: string; label: string }) => (
    <button
      onClick={() => setActiveTab(tab as any)}
      style={{
        padding: "4px 12px", fontSize: 12, border: "none", borderRadius: 4, cursor: "pointer",
        background: activeTab === tab ? "#4361ee" : "#e8e8e8",
        color: activeTab === tab ? "#fff" : "#333",
      }}
    >
      {label}
    </button>
  );

  const Toggle = ({ checked, onChange }: { checked: boolean; onChange: () => void }) => (
    <label style={{ position: "relative", width: 44, height: 24, cursor: "pointer" }}>
      <input type="checkbox" checked={checked} onChange={onChange} style={{ display: "none" }} />
      <span style={{
        position: "absolute", top: 0, left: 0, right: 0, bottom: 0,
        background: checked ? "#4361ee" : "#ccc", borderRadius: 12, transition: "background 0.2s",
      }}>
        <span style={{
          position: "absolute", top: 2, left: checked ? 22 : 2,
          width: 20, height: 20, background: "#fff", borderRadius: "50%", transition: "left 0.2s",
        }} />
      </span>
    </label>
  );

  return (
    <div style={{ height: "100vh", display: "flex", flexDirection: "column", background: "#f5f5f5" }}>
      {/* Header */}
      <div style={{ padding: "12px 16px", background: "#fff", borderBottom: "1px solid #e8e8e8" }}>
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <h1 style={{ margin: 0, fontSize: 18, fontWeight: 600, color: "#1a1a2e" }}>{t.title}</h1>
          <div style={{ flex: 1 }} />
          <div style={{ display: "flex", gap: 4, marginRight: 8 }}>
            <TabButton tab="records" label={t.recordsTab} />
            <TabButton tab="settings" label={t.settingsTab} />
            <TabButton tab="sync" label={t.syncTab} />
          </div>
          {activeTab === "records" && (
            <button onClick={handleClearAll} style={{ padding: "4px 12px", fontSize: 12, background: "#ff4757", color: "#fff", border: "none", borderRadius: 4, cursor: "pointer" }}>
              {t.clearAll}
            </button>
          )}
        </div>

        {activeTab === "records" && (
          <div style={{ display: "flex", gap: 8, marginTop: 12 }}>
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleSearch()}
              placeholder={t.searchPlaceholder}
              style={{ flex: 1, padding: "8px 12px", border: "1px solid #d0d0d0", borderRadius: 6, fontSize: 14, outline: "none" }}
            />
            {isSearching ? (
              <button onClick={handleClearSearch} style={{ padding: "8px 16px", background: "#555", color: "#fff", border: "none", borderRadius: 6, cursor: "pointer" }}>
                {t.clear}
              </button>
            ) : (
              <button onClick={handleSearch} style={{ padding: "8px 16px", background: "#4361ee", color: "#fff", border: "none", borderRadius: 6, cursor: "pointer" }}>
                {t.search}
              </button>
            )}
          </div>
        )}
      </div>

      {/* Content */}
      {activeTab === "records" && (
        <div style={{ flex: 1, overflow: "auto", padding: 8 }}>
          {records.length === 0 ? (
            <div style={{ textAlign: "center", padding: 40, color: "#999" }}>
              {t.noRecords}
            </div>
          ) : (
            records.map((record) => (
              <div
                key={record.id}
                style={{
                  padding: 12, marginBottom: 6, background: "#fff", borderRadius: 8,
                  border: "1px solid #e8e8e8", cursor: "pointer", position: "relative",
                }}
              >
                <div style={{ display: "flex", alignItems: "flex-start", gap: 8 }}>
                  <div style={{ flex: 1, minWidth: 0 }}>
                    <div style={{
                      fontSize: 14, color: "#333", wordBreak: "break-all",
                      whiteSpace: "pre-wrap", maxHeight: 80, overflow: "hidden", lineHeight: 1.5,
                    }}>
                      {record.content}
                    </div>
                    <div style={{ marginTop: 6, fontSize: 12, color: "#999", display: "flex", gap: 12 }}>
                      <span>{formatTime(record.created_at)}</span>
                      {record.copy_count > 0 && <span>{t.uses} {record.copy_count} {t.times}</span>}
                      <span style={{ textTransform: "uppercase" }}>{record.record_type}</span>
                    </div>
                  </div>
                  <div style={{ display: "flex", gap: 4, flexShrink: 0 }}>
                    <button
                      onClick={(e) => { e.stopPropagation(); handleStar(record.id); }}
                      style={{ padding: "4px 8px", background: "none", border: "none", cursor: "pointer", fontSize: 16, color: record.is_starred ? "#f9a825" : "#ccc" }}
                      title={t.favorite}
                    >
                      {record.is_starred ? "★" : "☆"}
                    </button>
                    <button
                      onClick={(e) => { e.stopPropagation(); handlePaste(record.id); }}
                      style={{ padding: "4px 8px", background: "#4361ee", color: "#fff", border: "none", borderRadius: 4, cursor: "pointer", fontSize: 12 }}
                      title={t.paste}
                    >
                      {t.paste}
                    </button>
                    <button
                      onClick={(e) => { e.stopPropagation(); handleDelete(record.id); }}
                      style={{ padding: "4px 8px", background: "none", border: "none", cursor: "pointer", fontSize: 14, color: "#ccc" }}
                      title={t.delete}
                    >
                      ✕
                    </button>
                  </div>
                </div>
              </div>
            ))
          )}
        </div>
      )}

      {activeTab === "settings" && (
        <div style={{ flex: 1, overflow: "auto", padding: 16 }}>
          <div style={{ background: "#fff", borderRadius: 8, padding: 16, marginBottom: 12 }}>
            <h3 style={{ margin: "0 0 12px 0", fontSize: 15, fontWeight: 600 }}>{t.generalSettings}</h3>

            <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "10px 0", borderBottom: "1px solid #f0f0f0" }}>
              <div>
                <div style={{ fontSize: 14, color: "#333" }}>{t.autoStart}</div>
                <div style={{ fontSize: 12, color: "#999" }}>{t.autoStartDesc}</div>
              </div>
              <Toggle checked={autoStart} onChange={handleAutoStartToggle} />
            </div>

            <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "10px 0", borderBottom: "1px solid #f0f0f0" }}>
              <div>
                <div style={{ fontSize: 14, color: "#333" }}>{t.monitoring}</div>
                <div style={{ fontSize: 12, color: "#999" }}>{t.monitoringDesc}</div>
              </div>
              <Toggle checked={isMonitoring} onChange={handleToggleMonitor} />
            </div>

            <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "10px 0" }}>
              <div>
                <div style={{ fontSize: 14, color: "#333" }}>{t.language}</div>
                <div style={{ fontSize: 12, color: "#999" }}>{t.languageDesc}</div>
              </div>
              <div style={{ display: "flex", gap: 8 }}>
                <button onClick={() => handleLanguageChange("zh")} style={{
                  padding: "6px 16px", fontSize: 13, border: "1px solid",
                  borderColor: language === "zh" ? "#4361ee" : "#d0d0d0",
                  background: language === "zh" ? "#4361ee" : "#fff",
                  color: language === "zh" ? "#fff" : "#333", borderRadius: 6, cursor: "pointer",
                }}>
                  {t.chinese}
                </button>
                <button onClick={() => handleLanguageChange("en")} style={{
                  padding: "6px 16px", fontSize: 13, border: "1px solid",
                  borderColor: language === "en" ? "#4361ee" : "#d0d0d0",
                  background: language === "en" ? "#4361ee" : "#fff",
                  color: language === "en" ? "#fff" : "#333", borderRadius: 6, cursor: "pointer",
                }}>
                  {t.english}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {activeTab === "sync" && (
        <div style={{ flex: 1, overflow: "auto", padding: 16 }}>
          <div style={{ background: "#fff", borderRadius: 8, padding: 16, marginBottom: 12 }}>
            <h3 style={{ margin: "0 0 12px 0", fontSize: 15, fontWeight: 600 }}>{t.sync}</h3>
            <div style={{ fontSize: 12, color: "#999", marginBottom: 12 }}>{t.syncDesc}</div>

            <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "10px 0", borderBottom: "1px solid #f0f0f0" }}>
              <div>
                <div style={{ fontSize: 14, color: "#333" }}>
                  {isSyncRunning ? t.statusRunning : t.statusStopped} {localIp ? `(${localIp}:${syncPort})` : ""}
                </div>
                <div style={{ fontSize: 12, color: isSyncRunning ? "#22c55e" : "#999" }}>
                  {isSyncRunning ? t.statusRunning : t.statusStopped}
                </div>
              </div>
              <Toggle checked={isSyncRunning} onChange={handleToggleSync} />
            </div>

            <div style={{ padding: "10px 0", borderBottom: "1px solid #f0f0f0" }}>
              <div style={{ fontSize: 14, color: "#333" }}>{t.syncPort}</div>
              <input
                type="number"
                value={syncPort}
                onChange={(e) => handleSyncPortChange(e.target.value)}
                style={{ marginTop: 4, padding: "6px 10px", border: "1px solid #d0d0d0", borderRadius: 6, fontSize: 14, width: 120, outline: "none" }}
              />
            </div>
          </div>

          <div style={{ background: "#fff", borderRadius: 8, padding: 16, marginBottom: 12 }}>
            <h3 style={{ margin: "0 0 12px 0", fontSize: 15, fontWeight: 600 }}>配对码</h3>
            <p style={{ fontSize: 12, color: "#666", marginBottom: 8 }}>显示本机 6 位配对码，在手机上输入此码完成配对</p>
            
            <div style={{ textAlign: "center", padding: "16px 0", background: "#f9fafb", borderRadius: 8, marginBottom: 12 }}>
              <div style={{ fontSize: 36, fontWeight: 700, letterSpacing: 8, color: "#4361ee", fontFamily: "monospace" }}>
                {myPairCode}
              </div>
            </div>

            <div style={{ borderBottom: "1px solid #f0f0f0", paddingBottom: 12, marginBottom: 12 }}>
              <div style={{ fontSize: 12, color: "#666", marginBottom: 8 }}>输入手机上的 6 位配对码</div>
              <div style={{ display: "flex", gap: 8 }}>
                <input
                  type="text"
                  value={peerPairCode}
                  onChange={(e) => {
                    const val = e.target.value.replace(/\D/g, "").slice(0, 6);
                    setPeerPairCode(val);
                  }}
                  placeholder="000000"
                  maxLength={6}
                  style={{ flex: 1, padding: "8px 12px", border: "1px solid #d0d0d0", borderRadius: 6, fontSize: 18, textAlign: "center", letterSpacing: 4, fontFamily: "monospace", outline: "none" }}
                />
                <button
                  onClick={handleVerifyPeerPairCode}
                  style={{ padding: "8px 20px", background: "#4361ee", color: "#fff", border: "none", borderRadius: 6, cursor: "pointer", fontSize: 14 }}
                >
                  验证
                </button>
              </div>
              {pairStatus && (
                <div style={{ marginTop: 8, fontSize: 12, color: pairStatus.includes("成功") ? "#22c55e" : "#ff4757" }}>
                  {pairStatus}
                </div>
              )}
            </div>

            {isPaired && (
              <div style={{ padding: 8, background: "#d4edda", borderRadius: 6, fontSize: 13, color: "#155724", textAlign: "center" }}>
                已配对
              </div>
            )}
          </div>

          <div style={{ background: "#fff", borderRadius: 8, padding: 16, marginBottom: 12 }}>
            <h3 style={{ margin: "0 0 12px 0", fontSize: 15, fontWeight: 600 }}>{t.sendToPeer}</h3>

            <div style={{ display: "flex", gap: 8, marginBottom: 8 }}>
              <div style={{ flex: 1 }}>
                <div style={{ fontSize: 12, color: "#999", marginBottom: 4 }}>{t.peerIp}</div>
                <input
                  type="text"
                  value={peerIp}
                  onChange={(e) => setPeerIp(e.target.value)}
                  placeholder="192.168.x.x"
                  style={{ width: "100%", padding: "6px 10px", border: "1px solid #d0d0d0", borderRadius: 6, fontSize: 14, outline: "none" }}
                />
              </div>
              <div style={{ width: 80 }}>
                <div style={{ fontSize: 12, color: "#999", marginBottom: 4 }}>{t.peerPort}</div>
                <input
                  type="number"
                  value={peerPort}
                  onChange={(e) => setPeerPort(parseInt(e.target.value) || 18910)}
                  style={{ width: "100%", padding: "6px 10px", border: "1px solid #d0d0d0", borderRadius: 6, fontSize: 14, outline: "none" }}
                />
              </div>
            </div>

            <textarea
              value={syncText}
              onChange={(e) => setSyncText(e.target.value)}
              placeholder={t.syncTextPlaceholder}
              rows={3}
              style={{ width: "100%", padding: "8px 12px", border: "1px solid #d0d0d0", borderRadius: 6, fontSize: 14, resize: "vertical", outline: "none", marginBottom: 8 }}
            />

            <div style={{ display: "flex", gap: 8 }}>
              <button
                onClick={handleSendToPeer}
                style={{ flex: 1, padding: "8px 16px", background: "#4361ee", color: "#fff", border: "none", borderRadius: 6, cursor: "pointer", fontSize: 13 }}
              >
                {t.sendToPeer}
              </button>
              <button
                onClick={handleRequestRecords}
                style={{ flex: 1, padding: "8px 16px", background: "#22c55e", color: "#fff", border: "none", borderRadius: 6, cursor: "pointer", fontSize: 13 }}
              >
                {t.requestRecords}
              </button>
            </div>

            {syncStatus && (
              <div style={{ marginTop: 8, fontSize: 12, color: syncStatus.includes("成功") || syncStatus.includes("Received") ? "#22c55e" : syncStatus.includes("失败") || syncStatus.includes("Failed") ? "#ff4757" : "#999" }}>
                {syncStatus}
              </div>
            )}
          </div>
        </div>
      )}

      {/* Quit Dialog */}
      {quitDialogOpen && (
        <div style={{
          position: "fixed", inset: 0, background: "rgba(0,0,0,0.5)",
          display: "flex", alignItems: "center", justifyContent: "center", zIndex: 1000,
        }}>
          <div style={{ background: "#fff", borderRadius: 12, padding: 24, width: 360, maxWidth: "90vw" }}>
            <h2 style={{ margin: "0 0 8px 0", fontSize: 18, fontWeight: 600 }}>{t.quitTitle}</h2>
            <p style={{ margin: "0 0 20px 0", fontSize: 14, color: "#666" }}>{t.quitMessage}</p>
            <div style={{ display: "flex", gap: 8 }}>
              <button
                onClick={() => handleQuitAction("minimize")}
                style={{ flex: 1, padding: "10px 16px", background: "#4361ee", color: "#fff", border: "none", borderRadius: 8, cursor: "pointer", fontSize: 14 }}
              >
                {t.minimize}
              </button>
              <button
                onClick={() => handleQuitAction("exit")}
                style={{ flex: 1, padding: "10px 16px", background: "#ff4757", color: "#fff", border: "none", borderRadius: 8, cursor: "pointer", fontSize: 14 }}
              >
                {t.exit}
              </button>
            </div>
            <button
              onClick={() => setQuitDialogOpen(false)}
              style={{ width: "100%", marginTop: 12, padding: "10px 16px", background: "#e8e8e8", color: "#333", border: "none", borderRadius: 8, cursor: "pointer", fontSize: 14 }}
            >
              {t.cancel}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
