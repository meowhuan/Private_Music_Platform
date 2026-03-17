# Private Music Platform

一个只属于自己的私人音乐空间：本地曲库 + 网易云 + 多端同步 + 歌词与播放队列。  
前端基于 Vite + Vue3 + UnoCSS，后端基于 Rust + Axum + MySQL。

## 功能概览
- 账号登录（JWT）与多用户数据隔离
- 管理员注册用户（仅站长可操作，邮箱：meowhuan@meowra.cn）
- 用户修改自己的密码（管理员需通过环境变量修改）
- 管理员重置用户密码
- 本地音乐上传（读取标签/时长/内嵌歌词）
- 上传封面或自动从网易云搜索封面
- 网易云搜索、歌单导入、播放与歌词
- 播放队列（顺序/循环/随机）、拖拽排序、队列搜索
- 歌词高亮与滚动、缓存与预加载
- 设备与同步（方案B：设备注册、同步状态、播放进度/缓存信息）
- 播放数据统计与趋势

## 目录结构
```
.
├─ backend/              # Rust + Axum 后端
├─ src/                  # 前端源码
├─ public/               # 静态资源
├─ .env.example          # 前端环境变量示例
└─ README.md
```

## 前端启动
1. 复制环境变量
```
cp .env.example .env
```
2. 配置 `VITE_API_BASE`
```
VITE_API_BASE=http://127.0.0.1:8080
```
3. 启动
```
pnpm install
pnpm dev
```

## 后端启动
1. 复制环境变量
```
cp backend/.env.example backend/.env
```
2. 配置 `DATABASE_URL` 等
```
DATABASE_URL=mysql://USER:PASSWORD@HOST:3306/Private_Music_Platform
JWT_SECRET=replace_with_long_random_string
ADMIN_USERNAME=admin
ADMIN_PASSWORD=replace_with_strong_password
MUSIC_STORAGE_DIR=./storage/music
NETEASE_BASE_URL=https://your_neteasemusicapi.com
BIND_ADDR=0.0.0.0:8080
```
3. 运行
```
cd backend
cargo run
```
后端启动时会自动执行 `sqlx::migrate!()` 迁移。

### 管理员密码变更说明
管理员密码不提供接口修改，请通过环境变量 `ADMIN_PASSWORD` 调整。  
启动时会自动同步该密码到数据库。  
若变更后无法登录，请确认已重启后端。

## 密码相关接口
- `POST /api/users/password`（用户修改自己的密码）
- `POST /api/users/password/reset`（管理员重置其他用户密码）

## 上传封面
上传音乐时可附带封面文件字段 `cover`（image/*）。  
未上传时会根据歌曲标题与艺人自动从网易云搜索封面。

## 设备与同步
### 管理端注册设备（仅管理员可用）
页面：设备与同步  
创建设备后会返回一次性 `device_token`，请保存并发给设备端。

### 设备上报（设备端）
```
POST /api/devices/:id/report
Header: x-device-token: <token>
Body(JSON):
{
  "status": "在线",
  "cache_size_mb": 1200,
  "synced_tracks": 356,
  "playing_title": "Lemon",
  "playing_artist": "米津玄師",
  "playback_progress": 42,
  "synced": true
}
```
上报会刷新 `last_seen_at`，若 `synced=true` 则更新 `last_sync_at`。

## 本地上传与歌词
- 上传音频时会尝试读取标签（标题/艺人/专辑）并写入数据库  
- 支持同名歌词 `.lrc/.txt` 或内嵌歌词  
- 本地音频文件保存到 `storage/music`
- 本地歌词文件会保存到 `storage/music/lyrics/{track_id}.lrc`

## 常用接口（前端使用）
- `POST /api/auth/login`
- `POST /api/users`（管理员创建用户）
- `GET /api/now-playing`
- `GET /api/queue`
- `POST /api/queue/add`
- `POST /api/queue/replace`
- `POST /api/queue/reorder`
- `GET /api/library`
- `GET /api/playlists`
- `GET /api/playlists/:id`
- `POST /api/netease/*` 系列代理

## 移动端适配
移动端采用响应式自适应，不影响 PC 布局。  
小屏会启用底部悬浮导航与紧凑间距。

## 部署流程（简版）
1. 准备 MySQL 数据库并创建 `Private_Music_Platform`
2. 配置后端 `.env` 并启动 `backend`
3. 配置前端 `.env` 并启动 `pnpm dev` 或构建静态站点
4. （可选）Nginx 反向代理后端 `/api` 与前端静态资源

## 说明
本项目为私人用途构建，仅限个人使用与自部署。  
网易云接口通过自建 API 代理访问，遵循后端配置的 `NETEASE_BASE_URL`。

## 开源许可
- 代码：AGPL-3.0
- 设计与内容：CC BY-NC-SA 4.0
