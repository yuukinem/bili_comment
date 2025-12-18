# B站评论助手

<p align="center">
  <img src="src-tauri/icons/icon.png" width="128" height="128" alt="B站评论助手">
</p>

<p align="center">
  <strong>一款跨平台的B站视频批量评论工具</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-blue" alt="Platform">
  <img src="https://img.shields.io/badge/Tauri-2.0-orange" alt="Tauri">
  <img src="https://img.shields.io/badge/Nuxt-4-green" alt="Nuxt">
  <img src="https://img.shields.io/badge/Rust-1.70+-red" alt="Rust">
</p>

---

## 功能特性

- **扫码登录** - 使用B站APP扫码安全登录
- **视频搜索** - 关键词搜索视频，支持多种排序方式
- **批量评论** - 勾选多个视频后批量发送评论
- **快速评论** - 单个视频一键发送预设评论
- **评论模板** - 管理常用评论模板，提高效率
- **任务进度** - 实时显示批量评论进度
- **跨平台** - 支持 macOS、Windows、Linux

## 截图预览

<!-- 添加应用截图 -->

## 技术栈

| 组件 | 技术 |
|------|------|
| 前端框架 | Nuxt 4 + Vue 3 |
| UI 组件库 | Naive UI |
| 状态管理 | Pinia |
| 桌面框架 | Tauri 2 |
| 后端语言 | Rust |
| HTTP 客户端 | Reqwest |

## 安装使用

### 下载安装

从 [Releases](../../releases) 页面下载对应平台的安装包：

| 平台 | 文件格式 |
|------|----------|
| macOS (Apple Silicon) | `.dmg` (aarch64) |
| macOS (Intel) | `.dmg` (x64) |
| Windows | `.msi` / `.exe` |
| Linux | `.deb` / `.AppImage` |

### 从源码构建

#### 环境要求

- Node.js 20+
- pnpm 9+
- Rust 1.70+
- 系统依赖（Linux）:
  ```bash
  sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev
  ```

#### 构建步骤

```bash
# 克隆项目
git clone https://github.com/yuukinem/bili_comment.git
cd bili_comment

# 安装依赖
pnpm install

# 开发模式
pnpm tauri dev

# 构建发布版本
pnpm tauri build
```

## 使用说明

### 1. 登录

启动应用后，使用B站APP扫描二维码登录。

### 2. 搜索视频

在搜索框输入关键词，点击搜索或按回车。支持以下排序方式：
- 综合排序
- 最多播放
- 最新发布
- 最多弹幕
- 最多收藏

### 3. 批量评论

1. 勾选要评论的视频（可点击"全选"）
2. 点击"批量评论"按钮
3. 输入评论内容或选择模板
4. 确认发送

### 4. 评论模板

点击右上角用户头像 → 模板管理，可以：
- 添加新模板
- 编辑现有模板
- 删除模板

## 注意事项

- 评论间隔默认为 5 秒，避免触发B站风控
- 请遵守B站社区规范，文明评论
- 本工具仅供学习交流使用

## 项目结构

```
bili_comment/
├── app/                    # Nuxt 前端
│   ├── pages/              # 页面
│   ├── stores/             # Pinia 状态管理
│   ├── components/         # Vue 组件
│   └── types/              # TypeScript 类型
├── src-tauri/              # Tauri/Rust 后端
│   ├── src/
│   │   ├── api/            # B站 API 封装
│   │   ├── commands/       # Tauri Commands
│   │   ├── models/         # 数据模型
│   │   └── storage/        # 数据持久化
│   └── icons/              # 应用图标
├── .github/workflows/      # CI/CD 配置
└── nuxt.config.ts          # Nuxt 配置
```

## 开发相关

### 日志输出

开发模式下，终端会显示详细日志：

```
🔍 搜索视频: keyword=xxx, page=1...
✅ 搜索成功: 找到 20 条结果...
💬 发送评论: aid=xxx, 内容="xxx"
✅ 评论成功: aid=xxx, rpid=xxx
```

### API 端点

| 功能 | 端点 |
|------|------|
| 二维码登录 | `passport.bilibili.com/x/passport-login/web/qrcode/*` |
| 用户信息 | `api.bilibili.com/x/web-interface/nav` |
| 视频搜索 | `api.bilibili.com/x/web-interface/search/type` |
| 发送评论 | `api.bilibili.com/x/v2/reply/add` |

## License

MIT License

## 免责声明

本项目仅供学习和研究使用，请勿用于任何违反B站用户协议的行为。使用本工具产生的任何后果由使用者自行承担。
