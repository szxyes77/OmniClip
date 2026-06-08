# OmniClip – 智能剪贴板管理器

> 100% 本地离线运行 · 局域网剪贴板同步 · 零云服务依赖

[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Android-blue)
![Tauri](https://img.shields.io/badge/Tauri-v2-FFC131?logo=tauri)
![React](https://img.shields.io/badge/React-18-61DAFB?logo=react)
![Flutter](https://img.shields.io/badge/Flutter-3-02569B?logo=flutter)
![Rust](https://img.shields.io/badge/Rust-2021-DEA584?logo=rust)

---

## 简介

OmniClip 是一款跨平台剪贴板管理工具，包含 **桌面端 (Tauri + Rust)** 和 **移动端 (Flutter)**，实现**自动剪贴板历史记录、局域网实时同步、搜索与一键粘贴**。所有数据存储在本地设备，永不离开你的网络。

### 它解决的痛点

- **剪贴板覆盖丢失** — 复制新内容后，旧内容永久消失
- **设备间剪贴板隔离** — 手机和电脑在同一局域网却无法共享剪贴板
- **系统剪贴板无历史** — Windows / macOS / Linux / Android 均不提供搜索或历史记录
- **敏感信息不上云** — 多数剪贴板同步工具依赖云服务，存在隐私泄露风险

---

## ✨ 功能特性

### 桌面端 (PC)

- **自动记录** — 文本历史记录，自动捕获剪贴板变化
- **极速搜索** — 全文模糊搜索，毫秒级响应
- **星标收藏** — 一键标记常用片段，快速访问
- **悬浮窗口** — 半透明窗口，快速唤起
- **系统托盘** — 常驻后台，右键菜单快速访问
- **局域网同步** — 与手机端实时同步剪贴板内容
- **全局快捷键** — 秒唤悬浮窗，快捷键可自定义

### 移动端 (Android)

- **实时同步** — 通过 SSE (Server-Sent Events) 实时接收 PC 剪贴板更新
- **发送到电脑** — 一键将手机剪贴板内容发送到 PC 系统剪贴板
- **记录管理** — 浏览、搜索、星标、删除同步记录
- **设备配对** — 通过配对码快速连接 PC

### 同步架构

```
┌─────────────┐                    ┌─────────────┐
│手机 (Flutter)│◄── SSE 长连接 ──── │  PC (Tauri) │
│             │                    │             │
│  实时接收推送│◄─clipboard-update  │ 剪贴板监控   │
│  发送文本到PC│─POST /api/sync──►  │ 存储+广播    │
└─────────────┘                    └─────────────┘

传输协议: HTTP (端口 18911) + SSE 实时推送
设备发现: mDNS 广播 (端口 53531)
数据格式: JSON (明文，仅限局域网)
```

---

## 🛠️ 技术栈

| 层级         | 技术选型                                          |
|--------------|---------------------------------------------------|
| 桌面框架     | [Tauri v2](https://v2.tauri.app/)                |
| 桌面前端     | React 18 + TypeScript + Vite + Tailwind CSS      |
| 桌面后端     | Rust (SQLite + SSE + mDNS)                       |
| 移动端       | Flutter (Dart)                                   |
| 数据库       | SQLite（Rusqlite 驱动）                           |
| 实时推送     | SSE (Server-Sent Events) + crossbeam-channel     |
| 设备发现     | mDNS UDP 广播                                    |
| 内容哈希     | SHA-256 (去重)                                   |

---

## 🚀 快速开始

### 桌面端 (PC)

#### 前置要求

| 工具     | 最低版本  | 安装方式                            |
|----------|-----------|-------------------------------------|
| Node.js  | ≥ 18      | [nodejs.org](https://nodejs.org)    |
| Rust     | ≥ 1.70    | [rustup.rs](https://rustup.rs)      |

**Windows 额外要求**：安装 [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)（包含 C++ 生成工具）

#### 安装与运行

```bash
# 1. 克隆仓库
git clone https://github.com/szxyes77/OmniClip.git
cd OmniClip

# 2. 安装依赖
npm install

# 3. 开发模式（热重载）
npm run tauri dev

# 4. 构建生产安装包
npm run tauri build
```

构建完成后，安装包位于 `src-tauri/target/release/bundle/` 目录下：
- Windows NSIS 安装包：`nsis/OmniClip_x.x.x_x64-setup.exe`

### 移动端 (Android)

#### 前置要求

- Flutter SDK ≥ 3.0
- Android Studio / VS Code with Flutter 插件
- Android SDK 或真机

#### 安装与运行

```bash
# 1. 进入移动端目录
cd omniclip_mobile

# 2. 获取依赖
flutter pub get

# 3. 运行（连接真机或启动模拟器）
flutter run
```

### 使用同步功能

1. **启动 PC 端** — 运行 OmniClip，同步服务自动启动（默认端口 18911）
2. **打开手机端** — 输入 PC 的 IP 地址和端口（默认 18911）
3. **自动同步** — PC 复制内容后，手机端通过 SSE 实时接收；手机端发送内容，PC 剪贴板立即更新

---

## 📂 项目结构

```
OmniClip/
├── src/                          # React 桌面端前端（TypeScript）
│   ├── api/                      # Tauri 命令封装
│   └── App.tsx / main.tsx        # 入口文件
│
├── src-tauri/                    # Rust 桌面端后端
│   ├── src/
│   │   ├── clipboard/            # 剪贴板监听与处理
│   │   ├── database/             # SQLite 数据库操作
│   │   ├── sync/                 # 局域网同步（SSE + mDNS + HTTP）
│   │   ├── tray/                 # 系统托盘
│   │   ├── lib.rs                # Tauri 命令入口与应用生命周期
│   │   └── main.rs               # 程序入口
│   ├── Cargo.toml                # Rust 依赖
│   └── tauri.conf.json           # Tauri 配置
│
├── omniclip_mobile/              # Flutter 移动端应用
│   ├── lib/
│   │   ├── api/                  # HTTP API 客户端
│   │   ├── services/             # SSE 客户端
│   │   ├── ui/                   # 页面组件
│   │   └── utils/                # 配置管理
│   └── pubspec.yaml              # Flutter 依赖
│
├── package.json
└── README.md
```

---

## 📖 使用说明

### 桌面端

#### 首次启动

1. 运行 OmniClip 后，应用自动启动并**隐藏至系统托盘**，开始后台监听剪贴板
2. 每次复制内容自动存储，重复内容自动去重（SHA-256 哈希比对）
3. 同步服务自动启动，监听端口 18911

#### 快捷键

| 快捷键      | 功能             |
|-------------|------------------|
| `Alt+V`     | 唤起悬浮窗口     |

### 移动端

#### 记录页面

- 自动显示 PC 端同步的剪贴板历史
- PC 复制新内容后，手机端实时推送通知并刷新
- 支持星标、删除、复制操作

#### 发送页面

- 输入 PC 的 IP 地址和端口（默认 18911）
- 选择一条剪贴板记录，点击"发送到设备"
- PC 系统剪贴板立即更新为该内容

### 同步端口说明

| 端口   | 协议   | 用途                      |
|--------|--------|---------------------------|
| 18910  | TCP    | 传统 TCP 直连（可选）      |
| 18911  | HTTP   | HTTP API + SSE 实时推送    |
| 53531  | UDP    | mDNS 设备发现              |

### 数据存储

- 数据库文件位于系统应用数据目录
  - Windows: `%APPDATA%\OmniClip\omniclip.db`
- 同步仅在局域网内工作，数据不经过任何外部服务器

---

## 🧑‍💻 开发指南

### 添加新的同步端点

1. 在 `src-tauri/src/sync/server.rs` 的 `handle_http_request` 中添加新的路由处理
2. 在 `omniclip_mobile/lib/api/client.dart` 中添加对应的 API 方法

### 修改 SSE 事件格式

SSE 事件格式：
```
event: clipboard-update
data: {"id":"...","content":"...","recordType":"text",...}

```

修改 [EventBroadcaster](file:///e:/vscode_project/OmniClip/src-tauri/src/sync/server.rs#L20-L68) 的 `broadcast` 方法可更改事件类型。

---

## 🗺️ 路线图

- [ ] 端到端加密传输
- [ ] iOS 移动端支持
- [ ] macOS 桌面端支持
- [ ] 文件/图片同步
- [ ] 多设备群组管理
- [ ] Web 端访问

---

## 📝 许可证

[MIT License](LICENSE)

Copyright (c) 2024-2026 szxyes77

---

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建功能分支（`git checkout -b feature/your-feature`）
3. 提交更改（`git commit -m 'feat: add your feature'`）
4. 推送到分支（`git push origin feature/your-feature`）
5. 提交 Pull Request

---

<p align="center">Made with ❤️ using Tauri + React + Rust + Flutter</p>
