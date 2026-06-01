# CHANGELOG

All notable changes to OmniClip will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-06-01

### Added
- 全自动剪贴板监听（文本/图片/文件路径），800ms 轮询捕获
- 历史记录加密存储（AES-256-GCM + PBKDF2-HMAC-SHA256）
- 全文模糊搜索，支持中文拼音全拼/首字母/子串匹配
- 星标收藏与多标签分类管理
- 悬浮粘贴窗口（Alt+V 全局快捷键唤出）
- 系统托盘常驻后台，右键菜单交互
- 剪贴板监听暂停/恢复功能
- 关闭主窗口隐藏至托盘（不退出程序）
- 加密 JSON 数据导入/导出
- 自动清理策略（保留天数/最大条数/星标保留）
- 系统原生对话框（文件选择/保存）
- 开机骨架屏加载动画
- 中英文国际化支持

### Changed
- 生产构建自动移除 console.log/debugger
- CSP 策略限制仅加载本地资源
- 窗口配置优化（最小尺寸/禁止拖拽文件入窗口）

### Security
- 所有数据存储端对端加密
- 机器指纹绑定密钥派生
- 零网络请求，完全离线运行
