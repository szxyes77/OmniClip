# OmniClip – 智能剪贴板历史管理器

> 100% 本地离线运行 · 零云服务依赖 · 端到端加密存储

[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)
![Tauri](https://img.shields.io/badge/Tauri-v2-FFC131?logo=tauri)
![React](https://img.shields.io/badge/React-18-61DAFB?logo=react)
![TypeScript](https://img.shields.io/badge/TypeScript-5.5-3178C6?logo=typescript)
![Rust](https://img.shields.io/badge/Rust-2021-DEA584?logo=rust)

---

## 简介

OmniClip 是一款基于 **Tauri v2** 构建的跨平台桌面剪贴板管理器，结合 Rust 后端与 React 前端，实现**全自动剪贴板历史记录、加密存储、模糊搜索与一键粘贴**，所有数据永不离开你的设备。

### 它解决的痛点

- **剪贴板覆盖丢失** — 复制新内容后，旧内容永久消失
- **系统剪贴板无历史** — Windows / macOS / Linux 均不提供搜索或历史记录
- **搜索能力弱** — 无法通过拼音、关键词快速定位曾经复制的内容
- **敏感信息不上云** — 多数剪贴板工具依赖云服务，存在隐私泄露风险

---

## ✨ 功能特性

- 📋 **自动记录** — 文本、图片、文件路径历史，800ms 轮询自动捕获
- 🔍 **极速搜索** — 全文模糊搜索，支持中文全拼 / 首字母 / 子串匹配，毫秒级响应
- ⭐ **星标收藏** — 一键标记常用片段，快速访问
- 🏷️ **标签分类** — 多标签管理，随机柔和配色，过滤筛选一目了然
- 🪟 **悬浮小窗** — 毛玻璃半透明窗口，一键唤起无需切换当前工作区
- 🔒 **绝对隐私** — AES-256-GCM 加密存储，机器指纹 + 可选主密码，永不联网
- ⌨️ **全局快捷键** — Alt+V 秒唤悬浮窗，快捷键可自定义
- 🎨 **系统托盘** — 常驻后台，右键菜单快速访问

---

## 📸 截图

### 主界面

![主界面](screenshots/main.png)
> *搜索栏 + 历史虚拟列表 + 标签侧边栏*

### 悬浮粘贴窗口

![悬浮窗](screenshots/overlay.png)
> *毛玻璃背景 + 点击穿透 + 键盘导航*

### 搜索演示

![搜索](screenshots/search.png)
> *中文拼音搜索 "jiantieban" → 匹配 "剪贴板"*

---

## 🛠️ 技术栈

| 层级         | 技术选型                                          |
|--------------|---------------------------------------------------|
| 桌面框架     | [Tauri v2](https://v2.tauri.app/)                |
| 前端         | React 18 + TypeScript + Vite + Tailwind CSS      |
| 状态管理     | [Zustand](https://github.com/pmndrs/zustand)      |
| 虚拟列表     | [react-window](https://github.com/bvaughn/react-window) |
| 数据库       | SQLite（Rusqlite 驱动）                           |
| 搜索索引     | Rust 内存倒排索引 + pinyin crate 拼音转换          |
| 加密         | AES-256-GCM + PBKDF2-HMAC-SHA256（100,000 次迭代） |
| 粘贴模拟     | [enigo](https://github.com/enigo-rs/enigo)        |

---

## 🚀 快速开始

### 前置要求

| 工具     | 最低版本  | 安装方式                            |
|----------|-----------|-------------------------------------|
| Node.js  | ≥ 18      | [nodejs.org](https://nodejs.org)    |
| Rust     | ≥ 1.70    | [rustup.rs](https://rustup.rs)      |
| npm/pnpm | npm ≥ 9   | Node.js 自带或 [pnpm](https://pnpm.io) |

### 安装与运行

```bash
# 1. 克隆仓库
git clone https://github.com/szxyes77/OmniClip.git
cd OmniClip

# 2. 安装依赖
npm install
# 或使用 pnpm:  pnpm install

# 3. 开发模式（热重载）
npm run tauri:dev

# 4. 构建生产安装包
npm run tauri:build
```

构建完成后，安装包位于 `src-tauri/target/release/bundle/` 目录下。

---

## 📂 项目结构

```
OmniClip/
├── src/                          # React 前端（TypeScript）
│   ├── api/                      # Tauri 命令封装
│   ├── components/               # UI 组件
│   │   └── main/                 # 主界面组件
│   ├── i18n/                     # 国际化（中/英文）
│   ├── hooks/                    # 自定义 Hooks
│   ├── store/                    # Zustand 状态管理
│   ├── types/                    # TypeScript 类型定义
│   ├── utils/                    # 工具函数
│   └── App.tsx / main.tsx        # 入口文件
│
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── clipboard/            # 剪贴板监听与处理
│   │   ├── config/               # 设置管理
│   │   ├── database/             # SQLite 数据库操作
│   │   ├── encryption/           # AES 加密模块
│   │   ├── search/               # 拼音搜索索引
│   │   ├── overlay/              # 悬浮窗口逻辑
│   │   ├── tray/                 # 系统托盘
│   │   ├── data_export.rs        # 加密导入/导出
│   │   └── lib.rs                # Tauri 命令入口
│   ├── Cargo.toml                # Rust 依赖
│   └── tauri.conf.json           # Tauri 配置
│
├── package.json
├── tailwind.config.ts
└── README.md
```

---

## 📖 使用说明

### 首次启动

1. 运行 OmniClip 后，应用自动最小化到**系统托盘**，开始后台监听剪贴板
2. 每次复制内容自动加密存储，重复内容自动去重（SHA-256 哈希比对）

### 快捷键

| 快捷键      | 功能             |
|-------------|------------------|
| `Alt+V`     | 唤起悬浮粘贴窗口  |
| `↑ / ↓`     | 悬浮窗内导航条目  |
| `Enter`     | 粘贴选中条目      |
| `Escape`    | 关闭悬浮窗        |

> 全局快捷键可在设置页面自定义。

### 主窗口功能

- **搜索框**：输入关键词实时搜索，支持中文拼音、首字母、子串匹配
- **历史列表**：滚动加载，虚拟列表流畅渲染上千条记录
- **标签侧栏**：点击标签过滤，星标筛选独立开关
- **右键菜单**：复制纯文本、编辑标签、删除记录

### 数据存储与加密

- 数据库文件位于系统应用数据目录（路径因平台而异）
- 所有 `content` 字段使用 **AES-256-GCM** 加密存储
- 密钥通过机器指纹 + 可选主密码（PBKDF2 派生）生成
- 支持加密 JSON 导出 / 导入备份

---

## 🧑‍💻 开发指南

### 添加新的剪贴板类型

1. 在 `src-tauri/src/clipboard/models.rs` 中扩展 `ClipboardType` 枚举
2. 在 `monitor.rs` 的 `read_clipboard()` 中添加类型读取逻辑
3. 在 `lib.rs` 的 `paste_record` 命令中添加对应的粘贴处理分支
4. 在 `RecordCard.tsx` 中添加对应的图标渲染

### 扩展搜索索引

1. 修改 `src-tauri/src/search/engine.rs` 的 `build_index()` 方法
2. 在 `IndexedEntry` 结构体中添加新的索引字段
3. 在 `score_entry()` 中添加新的匹配规则和权重
4. 确保 `add_to_index()` 同步更新

### 打包多平台安装包

```bash
# Windows
npm run tauri:build:windows

# macOS (Intel)
npm run tauri:build:macos

# macOS (Apple Silicon)
npm run tauri:build:macos-arm

# Linux
npm run tauri:build:linux
```

跨平台编译需提前安装目标平台工具链：
```bash
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
rustup target add x86_64-unknown-linux-gnu
```

---

## 🗺️ 路线图

- [ ] 云同步（可选，端到端加密）
- [ ] 更多剪贴板类型支持（富文本、HTML）
- [ ] 浏览器扩展联动
- [ ] 剪贴板内容预览面板
- [ ] 自动清理策略 UI 可视化

---

## 📝 许可证

[MIT License](LICENSE)

Copyright (c) 2024 szxyes77

---

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建功能分支（`git checkout -b feature/your-feature`）
3. 提交更改（`git commit -m 'feat: add your feature'`）
4. 推送到分支（`git push origin feature/your-feature`）
5. 提交 Pull Request

---

<p align="center">Made with ❤️ using Tauri + React + Rust</p>
