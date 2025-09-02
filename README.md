# 🌟 Web Mini Note

[🇨🇳 中文](#中文文档) | [🇺🇸 English](#english-documentation)

---

## 中文文档

### 📝 简介

**Web Mini Note** 是一个极简、轻量级的在线笔记应用，使用 Rust 编写，编译后仅约 **3MB** 大小！支持实时保存、Markdown 渲染、文件上传、二维码分享等功能。

### ✨ 特性

- 🚀 **超轻量**: 编译后仅约 3MB，资源占用极低
- ⚡ **实时保存**: 自动保存笔记内容，无需手动操作
- 📱 **响应式设计**: 支持桌面和移动设备
- 🎨 **Markdown 支持**: 实时预览和渲染
- 📎 **文件上传**: 支持图片和文件上传（最大100MB）
- 🔗 **二维码分享**: 一键生成分享二维码
- 📜 **历史记录**: 查看最近访问的笔记
- 🎯 **快捷键**: 支持 Ctrl+S 保存等快捷操作
- 🔒 **隐私保护**: 本地部署，数据完全可控

### 🛠 技术栈

- **后端**: Rust + Axum 框架
- **前端**: 原生 JavaScript + HTML5
- **容器**: Docker 支持
- **依赖**: 极少的第三方依赖，编译产物小巧

### 🚀 快速开始

#### 方法一：Docker 运行（推荐）

```bash
# 拉取镜像并运行
docker run -d \
  --name web-mini-note \
  -p 8080:8080 \
  -v $(pwd)/data:/app/_tmp \
  -e PORT=8080 \
  -e SAVE_PATH=_tmp \
  -e FILE_LIMIT=100000 \
  -e SINGLE_FILE_SIZE_LIMIT=10240 \
  --restart always \
  5dprince/web-mini-note

# 访问 http://localhost:8080
```

#### 方法二：从源码构建

```bash
# 克隆项目
git clone <repository-url>
cd web-mini-note

# 构建（需要 Rust 环境）
cargo build --release

# 运行
./target/release/web-mini-note-rust
```

#### 方法三：Docker 构建

```bash
# 构建镜像
docker build -t web-mini-note .

# 运行容器
docker run -d \
  --name web-mini-note \
  -p 8080:8080 \
  -v $(pwd)/data:/app/_tmp \
  web-mini-note
```

### ⚙️ 环境变量配置

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| `PORT` | 8080 | 服务端口 |
| `SAVE_PATH` | _tmp | 笔记保存路径 |
| `FILE_LIMIT` | 100000 | 最大文件数量限制 |
| `SINGLE_FILE_SIZE_LIMIT` | 10240 | 单文件大小限制（字节） |
| `STATIC_ROOT` | . | 静态资源根目录 |

### 📖 使用说明

1. **创建笔记**: 访问首页自动生成随机笔记ID
2. **编辑内容**: 在文本框中输入内容，自动保存
3. **Markdown**: 点击 🔓 切换到预览模式
4. **文件上传**: 点击 ⤴ upload 上传图片或文件
5. **分享**: 点击 🔗 share 生成二维码分享
6. **历史**: 点击 📜 history 查看最近笔记
7. **快捷键**: 
   - `Ctrl+S`: 手动保存
   - `Ctrl+Enter`: 切换预览模式

### 🔧 API 接口

- `GET /` - 重定向到随机笔记
- `GET /{note}` - 获取笔记内容
- `POST /{note}` - 保存笔记内容
- `POST /upload` - 上传文件
- `GET /_tmp/{file}` - 访问上传的文件

### 📦 部署说明

项目支持多种部署方式：

1. **直接运行**: 编译后的二进制文件可直接运行
2. **Docker**: 使用提供的 Dockerfile 构建镜像
3. **反向代理**: 可配合 Nginx 等反向代理使用

### 🤝 贡献

欢迎提交 Issue 和 Pull Request！

---

## English Documentation

### 📝 Introduction

**Web Mini Note** is a minimalist, lightweight online note-taking application written in Rust, with a compiled size of only **~3MB**! It supports real-time saving, Markdown rendering, file uploads, QR code sharing, and more.

### ✨ Features

- 🚀 **Ultra Lightweight**: Only ~3MB compiled size with minimal resource usage
- ⚡ **Real-time Saving**: Auto-save note content without manual operations
- 📱 **Responsive Design**: Supports both desktop and mobile devices
- 🎨 **Markdown Support**: Real-time preview and rendering
- 📎 **File Upload**: Support image and file uploads (max 100MB)
- 🔗 **QR Code Sharing**: One-click QR code generation for sharing
- 📜 **History**: View recently accessed notes
- 🎯 **Keyboard Shortcuts**: Support Ctrl+S save and other shortcuts
- 🔒 **Privacy Protection**: Self-hosted with full data control

### 🛠 Tech Stack

- **Backend**: Rust + Axum framework
- **Frontend**: Vanilla JavaScript + HTML5
- **Container**: Docker support
- **Dependencies**: Minimal third-party dependencies for small binary size

### 🚀 Quick Start

#### Method 1: Docker Run (Recommended)

```bash
# Pull and run the image
docker run -d \
  --name web-mini-note \
  -p 8080:8080 \
  -v $(pwd)/data:/app/_tmp \
  -e PORT=8080 \
  -e SAVE_PATH=_tmp \
  -e FILE_LIMIT=100000 \
  -e SINGLE_FILE_SIZE_LIMIT=10240 \
  --restart always \
  5dprince/web-mini-note

# Access http://localhost:8080
```

#### Method 2: Build from Source

```bash
# Clone the project
git clone <repository-url>
cd web-mini-note

# Build (requires Rust environment)
cargo build --release

# Run
./target/release/web-mini-note-rust
```

#### Method 3: Docker Build

```bash
# Build image
docker build -t web-mini-note .

# Run container
docker run -d \
  --name web-mini-note \
  -p 8080:8080 \
  -v $(pwd)/data:/app/_tmp \
  web-mini-note
```

### ⚙️ Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | 8080 | Service port |
| `SAVE_PATH` | _tmp | Notes save path |
| `FILE_LIMIT` | 100000 | Maximum file count limit |
| `SINGLE_FILE_SIZE_LIMIT` | 10240 | Single file size limit (bytes) |
| `STATIC_ROOT` | . | Static resources root directory |

### 📖 Usage

1. **Create Note**: Visit homepage to auto-generate random note ID
2. **Edit Content**: Type in the text area, auto-saves content
3. **Markdown**: Click 🔓 to switch to preview mode
4. **File Upload**: Click ⤴ upload to upload images or files
5. **Share**: Click 🔗 share to generate QR code for sharing
6. **History**: Click 📜 history to view recent notes
7. **Shortcuts**: 
   - `Ctrl+S`: Manual save
   - `Ctrl+Enter`: Toggle preview mode

### 🔧 API Endpoints

- `GET /` - Redirect to random note
- `GET /{note}` - Get note content
- `POST /{note}` - Save note content
- `POST /upload` - Upload file
- `GET /_tmp/{file}` - Access uploaded files

### 📦 Deployment

The project supports multiple deployment methods:

1. **Direct Run**: Compiled binary can run directly
2. **Docker**: Use provided Dockerfile to build image
3. **Reverse Proxy**: Can be used with Nginx or other reverse proxies

### 🤝 Contributing

Issues and Pull Requests are welcome!

---

## 📄 License

MIT License

## 🙏 Acknowledgments

Thanks to all contributors and the Rust community for making this lightweight note-taking solution possible!
