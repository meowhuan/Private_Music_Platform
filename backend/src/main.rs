use axum::{
  async_trait,
  extract::{DefaultBodyLimit, FromRequestParts, Multipart, Path, Query, State},
  http::{header, HeaderMap, HeaderValue, StatusCode},
  body::{Body, Bytes},
  middleware,
  response::IntoResponse,
  routing::{get, post},
  Json, Router
};
use chrono::{Duration as ChronoDuration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use argon2::{PasswordHasher, PasswordVerifier};
use serde_json::json;
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlPoolOptions, MySqlPool, Row};
use std::{collections::HashMap, path::{Path as StdPath, PathBuf}, sync::Arc};
use tokio::{fs, io::AsyncWriteExt};
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};
use tokio_util::io::ReaderStream;
use tower_http::{cors::{Any, CorsLayer}, trace::TraceLayer};
use uuid::Uuid;
use lofty::prelude::{AudioFile, ItemKey, TaggedFileExt};
use lofty::tag::Accessor;
use lofty::probe::Probe;

#[derive(Clone)]
struct AppState {
  db: MySqlPool,
  http: reqwest::Client,
  config: Arc<Config>
}

#[derive(Clone)]
struct Config {
  database_url: String,
  jwt_secret: String,
  admin_username: String,
  admin_password: String,
  music_storage_dir: PathBuf,
  netease_base_url: String,
  bind_addr: String,
  cors_origin: Option<String>,
  public_media_base: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
  sub: String,
  exp: usize
}

#[derive(Clone, Debug)]
struct AuthUser {
  username: String,
  user_id: String
}

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
  type Rejection = (StatusCode, Json<ErrorResponse>);

  async fn from_request_parts(parts: &mut axum::http::request::Parts, state: &AppState) -> Result<Self, Self::Rejection> {
    let auth_header = parts.headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok());
    let Some(token) = auth_header.and_then(|v| v.strip_prefix("Bearer ")) else {
      return Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse::new("Missing bearer token"))));
    };

    let decoding = DecodingKey::from_secret(state.config.jwt_secret.as_bytes());
    let data = decode::<Claims>(token, &decoding, &Validation::default())
      .map_err(|_| (StatusCode::UNAUTHORIZED, Json(ErrorResponse::new("Invalid token"))))?;

    let user_id: Option<(String,)> = sqlx::query_as("SELECT id FROM users WHERE username = ?")
      .bind(&data.claims.sub)
      .fetch_optional(&state.db)
      .await
      .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;
    let Some((user_id,)) = user_id else {
      return Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse::new("Invalid token"))));
    };

    Ok(AuthUser { username: data.claims.sub, user_id })
  }
}

#[derive(Serialize)]
struct ErrorResponse {
  message: String
}

impl ErrorResponse {
  fn new(message: &str) -> Self {
    Self { message: message.to_string() }
  }
}

#[derive(Deserialize)]
struct LoginRequest {
  username: String,
  password: String
}

#[derive(Deserialize)]
struct UserCreateRequest {
  username: String,
  password: String
}

#[derive(Deserialize)]
struct ChangePasswordRequest {
  current_password: String,
  new_password: String
}

#[derive(Deserialize)]
struct AdminResetPasswordRequest {
  username: String,
  new_password: String
}

#[derive(Serialize)]
struct LoginResponse {
  token: String,
  expires_in: i64,
  user: UserInfo
}

#[derive(Serialize)]
struct UserInfo {
  username: String,
  is_admin: bool
}

#[derive(Serialize)]
struct UserCreateResponse {
  username: String
}

#[derive(Serialize)]
struct NowPlayingResponse {
  id: String,
  title: String,
  artist: String,
  album: String,
  length: String,
  duration_seconds: Option<i32>,
  progress: i32,
  quality: String,
  mood: String,
  source: String,
  source_id: Option<String>,
  cover_url: Option<String>
}

#[derive(Serialize)]
struct QueueItem {
  queue_id: String,
  id: String,
  title: String,
  artist: String,
  time: String,
  duration_seconds: Option<i32>,
  source: String,
  source_id: Option<String>,
  cover_url: Option<String>
}

#[derive(Serialize)]
struct PlaylistItem {
  id: String,
  name: String,
  desc: String,
  count: i64,
  mood: String
}

#[derive(Serialize)]
struct PlaylistDetail {
  id: String,
  name: String,
  desc: String,
  mood: String,
  tracks: Vec<LibraryItem>
}

#[derive(Serialize)]
struct LibraryItem {
  id: String,
  title: String,
  artist: String,
  album: String,
  time: String,
  duration_seconds: Option<i32>,
  tag: String,
  source: String,
  source_id: Option<String>,
  cover_url: Option<String>
}

#[derive(Serialize)]
struct DeviceItem {
  id: String,
  name: String,
  status: String,
  desc: String,
  last_seen_at: Option<String>,
  last_sync_at: Option<String>,
  cache_size_mb: Option<i32>,
  synced_tracks: Option<i32>,
  playing_title: Option<String>,
  playing_artist: Option<String>,
  playback_progress: Option<i32>
}

#[derive(Serialize)]
struct DeviceCreateResponse {
  id: String,
  token: String
}

#[derive(Serialize)]
struct StatItem {
  label: String,
  value: String
}

#[derive(Deserialize)]
struct QueueReplaceRequest {
  playlist_id: Option<String>,
  items: Option<Vec<QueueAddItem>>
}

#[derive(Deserialize)]
struct QueueAddItem {
  track_id: Option<String>,
  source: Option<String>,
  source_id: Option<String>,
  title: Option<String>,
  artist: Option<String>,
  album: Option<String>,
  cover_url: Option<String>
}

#[derive(Deserialize)]
struct QueueAddRequest {
  items: Vec<QueueAddItem>
}

#[derive(Deserialize)]
struct QueueReorderRequest {
  order: Vec<String>
}

#[derive(Deserialize)]
struct PlaylistAddTrackRequest {
  track_id: Option<String>,
  source: Option<String>,
  source_id: Option<String>,
  title: Option<String>,
  artist: Option<String>,
  album: Option<String>,
  cover_url: Option<String>
}

#[derive(Deserialize)]
struct PlaylistCreateRequest {
  name: String,
  desc: Option<String>,
  mood: Option<String>
}

#[derive(Deserialize)]
struct PlaylistUpdateRequest {
  name: Option<String>,
  desc: Option<String>,
  mood: Option<String>
}

#[derive(Deserialize)]
struct TrackUpdateRequest {
  title: Option<String>,
  artist: Option<String>,
  album: Option<String>,
  tag: Option<String>,
  cover_url: Option<String>,
  duration_seconds: Option<i32>
}

#[derive(Deserialize)]
struct DeviceCreateRequest {
  name: String,
  desc: Option<String>
}

#[derive(Deserialize)]
struct DeviceUpdateRequest {
  status: Option<String>,
  desc: Option<String>,
  cache_size_mb: Option<i32>,
  synced_tracks: Option<i32>,
  playing_title: Option<String>,
  playing_artist: Option<String>,
  playback_progress: Option<i32>
}

#[derive(Deserialize)]
struct DeviceReportRequest {
  status: Option<String>,
  cache_size_mb: Option<i32>,
  synced_tracks: Option<i32>,
  playing_title: Option<String>,
  playing_artist: Option<String>,
  playback_progress: Option<i32>,
  synced: Option<bool>
}

#[derive(Deserialize)]
struct PlaybackEventRequest {
  track_id: Option<String>,
  source: Option<String>,
  source_id: Option<String>,
  title: Option<String>,
  artist: Option<String>,
  duration_seconds: Option<i64>
}

#[derive(Deserialize)]
struct PlaybackTrendQuery {
  days: Option<i64>
}

#[derive(Serialize)]
struct PlaybackTrendResponse {
  label: String,
  labels: Vec<String>,
  values: Vec<i64>
}

async fn cover_get(
  State(state): State<AppState>,
  Path(file): Path<String>
) -> Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
  if file.contains("..") || file.contains('/') || file.contains('\\') {
    return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse::new("Invalid cover file"))));
  }
  let cover_dir = state.config.music_storage_dir.join("covers");
  let path = cover_dir.join(&file);
  if !path.exists() {
    return Err((StatusCode::NOT_FOUND, Json(ErrorResponse::new("Cover not found"))));
  }
  let file = tokio::fs::File::open(&path)
    .await
    .map_err(|_| (StatusCode::NOT_FOUND, Json(ErrorResponse::new("Cover not found"))))?;
  let stream = ReaderStream::new(file);
  let mut response = axum::response::Response::new(Body::from_stream(stream));
  if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
    let content_type = match ext.to_ascii_lowercase().as_str() {
      "png" => "image/png",
      "webp" => "image/webp",
      "gif" => "image/gif",
      _ => "image/jpeg"
    };
    response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
  }
  Ok(response)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  dotenvy::dotenv().ok();
  tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();

  let config = load_config()?;
  let pool = MySqlPoolOptions::new()
    .max_connections(10)
    .connect(&config.database_url)
    .await?;

  sqlx::migrate!().run(&pool).await?;
  ensure_storage_dir(&config.music_storage_dir).await?;
  let admin_user_id = ensure_admin_user(&pool, &config).await?;
  seed_if_empty(&pool, &admin_user_id).await?;
  backfill_user_id(&pool, &admin_user_id).await?;

  let http = reqwest::Client::builder()
    .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36")
    .timeout(std::time::Duration::from_secs(12))
    .build()?;

  let state = AppState {
    db: pool,
    http,
    config: Arc::new(config)
  };

  let cors = match &state.config.cors_origin {
    Some(origin) => {
      let mut layer = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .allow_credentials(true);
      let origin_value = HeaderValue::from_str(origin).unwrap_or_else(|_| HeaderValue::from_static("*"));
      layer = layer.allow_origin(tower_http::cors::AllowOrigin::exact(origin_value));
      layer
    }
    None => CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any)
  };

  let public_routes = Router::new()
    .route("/health", get(health))
    .route("/api/auth/login", post(login))
    .route("/api/cover/:file", get(cover_get))
    .route("/api/netease/search", get(netease_search))
    .route("/api/netease/song/detail", get(netease_song_detail))
    .route("/api/netease/song/url", get(netease_song_url))
    .route("/api/netease/song/url/v1", get(netease_song_url_v1))
    .route("/api/netease/lyric", get(netease_lyric))
    .route("/api/netease/lyric/new", get(netease_lyric_new))
    .route("/api/netease/playlist/detail", get(netease_playlist_detail))
    .route("/api/netease/playlist/track/all", get(netease_playlist_track_all))
    .route("/api/netease/stream", get(netease_stream))
    .route("/api/netease/login/cellphone", get(netease_login_cellphone))
    .route("/api/netease/login/email", get(netease_login_email))
    .route("/api/netease/login/status", get(netease_login_status).post(netease_login_status_post))
    .route("/api/netease/logout", get(netease_logout))
    .route("/api/netease/user/account", get(netease_user_account))
    .route("/api/netease/captcha/sent", get(netease_captcha_sent))
    .route("/api/netease/captcha/verify", get(netease_captcha_verify))
    .route("/api/netease/login/qr/key", get(netease_login_qr_key))
    .route("/api/netease/login/qr/create", get(netease_login_qr_create))
    .route("/api/netease/login/qr/check", get(netease_login_qr_check))
    .route("/api/devices/:id/report", post(device_report));

  let protected_routes = Router::new()
    .route("/api/users", post(user_create))
    .route("/api/users/password", post(user_change_password))
    .route("/api/users/password/reset", post(admin_reset_password))
    .route("/api/me", get(me))
    .route("/api/now-playing", get(now_playing))
    .route("/api/queue", get(queue_list))
    .route("/api/queue/replace", post(queue_replace))
    .route("/api/queue/clear", post(queue_clear))
    .route("/api/queue/add", post(queue_add))
    .route("/api/queue/reorder", post(queue_reorder))
    .route("/api/playlists", get(playlists))
    .route("/api/playlists", post(playlist_create))
    .route("/api/playlists/:id", get(playlist_detail))
    .route("/api/playlists/:id", post(playlist_update).delete(playlist_delete))
    .route("/api/playlists/:id/tracks", post(playlist_add_track))
    .route("/api/playlists/:id/tracks/:track_id", post(playlist_remove_track))
    .route("/api/library", get(library))
    .route("/api/tracks/:id", get(track_detail))
    .route("/api/tracks/:id", post(track_update).delete(track_delete))
    .route("/api/stream/:id", get(stream_track))
    .route("/api/local/url/:id", get(local_track_url))
    .route("/api/lyric/local/:id", get(local_lyric))
    .route("/api/devices", get(devices).post(device_create))
    .route("/api/devices/:id/sync", post(device_sync))
    .route("/api/devices/:id", post(device_update).delete(device_delete))
    .route("/api/stats", get(stats))
    .route("/api/playback/events", post(playback_event))
    .route("/api/playback/trend", get(playback_trend))
    .route(
      "/api/upload/music",
      post(upload_music).layer(DefaultBodyLimit::max(300 * 1024 * 1024))
    )
    .route_layer(middleware::from_extractor_with_state::<AuthUser, AppState>(state.clone()));

  let app = Router::new()
    .merge(public_routes)
    .merge(protected_routes)
    .with_state(state.clone())
    .layer(cors)
    .layer(TraceLayer::new_for_http());

  tracing::info!("listening on {}", state.config.bind_addr);
  let listener = tokio::net::TcpListener::bind(&state.config.bind_addr).await?;
  axum::serve(listener, app).await?;

  Ok(())
}

fn load_config() -> anyhow::Result<Config> {
  let database_url = std::env::var("DATABASE_URL")?;
  let jwt_secret = std::env::var("JWT_SECRET")?;
  let admin_username = std::env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());
  let admin_password = std::env::var("ADMIN_PASSWORD")?;
  let music_storage_dir = std::env::var("MUSIC_STORAGE_DIR").unwrap_or_else(|_| ".\\storage\\music".to_string());
  let netease_base_url = std::env::var("NETEASE_BASE_URL").unwrap_or_else(|_| "https://musicapi.meowra.cn".to_string());
  let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
  let cors_origin = std::env::var("CORS_ORIGIN").ok();
  let public_media_base = std::env::var("PUBLIC_MEDIA_BASE").ok();

  Ok(Config {
    database_url,
    jwt_secret,
    admin_username,
    admin_password,
    music_storage_dir: PathBuf::from(music_storage_dir),
    netease_base_url,
    bind_addr,
    cors_origin,
    public_media_base
  })
}

async fn ensure_storage_dir(dir: &StdPath) -> anyhow::Result<()> {
  if !dir.exists() {
    fs::create_dir_all(dir).await?;
  }
  Ok(())
}

async fn ensure_admin_user(pool: &MySqlPool, config: &Config) -> anyhow::Result<String> {
  let existing: Option<(String, String)> = sqlx::query_as("SELECT id, password_hash FROM users WHERE username = ?")
    .bind(&config.admin_username)
    .fetch_optional(pool)
    .await?;

  if let Some((id, password_hash)) = existing {
    if !verify_password(&password_hash, &config.admin_password) {
      let next_hash = hash_password(&config.admin_password)?;
      sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(next_hash)
        .bind(&id)
        .execute(pool)
        .await?;
    }
    return Ok(id);
  }

  let password_hash = hash_password(&config.admin_password)?;
  let user_id = Uuid::new_v4().to_string();
  sqlx::query("INSERT INTO users (id, username, password_hash) VALUES (?, ?, ?)")
    .bind(&user_id)
    .bind(&config.admin_username)
    .bind(password_hash)
    .execute(pool)
    .await?;

  Ok(user_id)
}

fn hash_password(password: &str) -> anyhow::Result<String> {
  let salt = argon2::password_hash::SaltString::generate(&mut rand_core::OsRng);
  let argon2 = argon2::Argon2::default();
  let hash = argon2
    .hash_password(password.as_bytes(), &salt)
    .map_err(|err| anyhow::anyhow!(err.to_string()))?
    .to_string();
  Ok(hash)
}

fn verify_password(hash: &str, password: &str) -> bool {
  let parsed = argon2::password_hash::PasswordHash::new(hash);
  if let Ok(parsed) = parsed {
    argon2::Argon2::default()
      .verify_password(password.as_bytes(), &parsed)
      .is_ok()
  } else {
    false
  }
}

async fn seed_if_empty(pool: &MySqlPool, admin_user_id: &str) -> anyhow::Result<()> {
  let track_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tracks WHERE user_id = ?")
    .bind(admin_user_id)
    .fetch_one(pool)
    .await?;
  if track_count.0 == 0 {
    let tracks = vec![
      ("星尘回响", "Meowra", "Private Archive Vol.2", 258, "Hi-Res · 24bit/96kHz", "夜行 / 沉浸", "收藏"),
      ("失眠海岸", "Meowra", "Private Archive", 222, "Hi-Res · 24bit/96kHz", "夜行 / 沉浸", "最近"),
      ("猫爪信号", "Aoi", "Soft Signal", 245, "FLAC", "甜雾", "收藏"),
      ("薄雾里", "Rin", "Fog", 312, "FLAC", "雨声", "最近"),
      ("凌晨三点", "Meowra", "Late Night", 241, "AAC", "夜行 / 沉浸", "离线")
    ];
    for (title, artist, album, duration, quality, mood, tag) in tracks {
      let id = Uuid::new_v4().to_string();
      sqlx::query("INSERT INTO tracks (id, title, artist, album, duration_seconds, quality, mood, tag, user_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(id)
        .bind(title)
        .bind(artist)
        .bind(album)
        .bind(duration)
        .bind(quality)
        .bind(mood)
        .bind(tag)
        .bind(admin_user_id)
        .execute(pool)
        .await?;
    }
  }

  let playlist_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM playlists WHERE user_id = ?")
    .bind(admin_user_id)
    .fetch_one(pool)
    .await?;
  if playlist_count.0 == 0 {
    let playlists = vec![
      ("夜航计划", "适合深夜工作与独处时段的流动感", "沉静"),
      ("软糖电台", "轻甜电子与梦核采样", "甜雾"),
      ("雨天窗口", "下雨时的钢琴与合成器", "雨声")
    ];
    for (name, desc, mood) in playlists {
      let id = Uuid::new_v4().to_string();
      sqlx::query("INSERT INTO playlists (id, name, description, mood, user_id) VALUES (?, ?, ?, ?, ?)")
        .bind(id)
        .bind(name)
        .bind(desc)
        .bind(mood)
        .bind(admin_user_id)
        .execute(pool)
        .await?;
    }
  }

  let device_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM devices WHERE user_id = ?")
    .bind(admin_user_id)
    .fetch_one(pool)
    .await?;
  if device_count.0 == 0 {
    let devices = vec![
      ("Meow Studio", "在线", "桌面端播放器"),
      ("iPad mini", "同步中", "离线缓存 1.2GB"),
      ("Android DAP", "待唤醒", "上次播放 2小时前")
    ];
    for (name, status, desc) in devices {
      let id = Uuid::new_v4().to_string();
      sqlx::query("INSERT INTO devices (id, name, status, description, user_id) VALUES (?, ?, ?, ?, ?)")
        .bind(id)
        .bind(name)
        .bind(status)
        .bind(desc)
        .bind(admin_user_id)
        .execute(pool)
        .await?;
    }
  }

  let now_playing_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM now_playing WHERE user_id = ?")
    .bind(admin_user_id)
    .fetch_one(pool)
    .await?;
  if now_playing_count.0 == 0 {
    let track_id: Option<(String,)> = sqlx::query_as("SELECT id FROM tracks WHERE user_id = ? LIMIT 1")
      .bind(admin_user_id)
      .fetch_optional(pool)
      .await?;
    if let Some((track_id,)) = track_id {
      let id = Uuid::new_v4().to_string();
      sqlx::query("INSERT INTO now_playing (id, track_id, progress_percent, started_at, user_id) VALUES (?, ?, ?, ?, ?)")
        .bind(id)
        .bind(track_id)
        .bind(42)
        .bind(Utc::now().naive_utc())
        .bind(admin_user_id)
        .execute(pool)
        .await?;
    }
  }

  let queue_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM queue WHERE user_id = ?")
    .bind(admin_user_id)
    .fetch_one(pool)
    .await?;
  if queue_count.0 == 0 {
    let track_ids: Vec<(String,)> = sqlx::query_as("SELECT id FROM tracks WHERE user_id = ? LIMIT 4")
      .bind(admin_user_id)
      .fetch_all(pool)
      .await?;
    for (idx, (track_id,)) in track_ids.into_iter().enumerate() {
      let id = Uuid::new_v4().to_string();
      sqlx::query("INSERT INTO queue (id, track_id, position, user_id) VALUES (?, ?, ?, ?)")
        .bind(id)
        .bind(track_id)
        .bind(idx as i32)
        .bind(admin_user_id)
        .execute(pool)
        .await?;
    }
  }

  let stat_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM usage_stats WHERE user_id = ?")
    .bind(admin_user_id)
    .fetch_one(pool)
    .await?;
  if stat_count.0 == 0 {
    let stats = vec![
      ("listening_time", "18h 24m"),
      ("today_listened", "42 首")
    ];
    for (key, value) in stats {
      let id = Uuid::new_v4().to_string();
      sqlx::query("INSERT INTO usage_stats (id, stat_key, stat_value, user_id) VALUES (?, ?, ?, ?) \
                   ON DUPLICATE KEY UPDATE stat_value = VALUES(stat_value), user_id = VALUES(user_id)")
        .bind(id)
        .bind(key)
        .bind(value)
        .bind(admin_user_id)
        .execute(pool)
        .await?;
    }
  }

  Ok(())
}

async fn backfill_user_id(pool: &MySqlPool, admin_user_id: &str) -> anyhow::Result<()> {
  let tables = [
    "tracks",
    "playlists",
    "playlist_tracks",
    "queue",
    "now_playing",
    "usage_stats",
    "devices",
    "play_events"
  ];
  for table in tables {
    let sql = format!("UPDATE {table} SET user_id = ? WHERE user_id IS NULL");
    sqlx::query(&sql)
      .bind(admin_user_id)
      .execute(pool)
      .await?;
  }
  Ok(())
}

async fn health() -> &'static str {
  "ok"
}

async fn login(State(state): State<AppState>, Json(payload): Json<LoginRequest>) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
  let row = sqlx::query("SELECT password_hash FROM users WHERE username = ?")
    .bind(&payload.username)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  let Some(row) = row else {
    return Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse::new("Invalid credentials"))));
  };

  let hash: String = row.try_get("password_hash").unwrap_or_default();
  if !verify_password(&hash, &payload.password) {
    return Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse::new("Invalid credentials"))));
  }

  let exp = (Utc::now() + ChronoDuration::days(7)).timestamp() as usize;
  let claims = Claims { sub: payload.username.clone(), exp };
  let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()))
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Token error"))))?;

  let username = payload.username;
  let is_admin = username == state.config.admin_username;
  Ok(Json(LoginResponse {
    token,
    expires_in: 7 * 24 * 3600,
    user: UserInfo { username, is_admin }
  }))
}

async fn me(State(state): State<AppState>, _user: AuthUser) -> Json<UserInfo> {
  Json(UserInfo {
    username: _user.username.clone(),
    is_admin: _user.username == state.config.admin_username
  })
}

async fn user_create(
  State(state): State<AppState>,
  user: AuthUser,
  Json(payload): Json<UserCreateRequest>
) -> Result<Json<UserCreateResponse>, (StatusCode, Json<ErrorResponse>)> {
  if user.username != state.config.admin_username {
    return Err((StatusCode::FORBIDDEN, Json(ErrorResponse::new("Forbidden"))));
  }
  if payload.username.trim().is_empty() || payload.password.trim().is_empty() {
    return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse::new("Username and password required"))));
  }

  let exists: Option<(String,)> = sqlx::query_as("SELECT id FROM users WHERE username = ?")
    .bind(payload.username.trim())
    .fetch_optional(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;
  if exists.is_some() {
    return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse::new("User already exists"))));
  }

  let password_hash = hash_password(payload.password.trim())
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Password hash failed"))))?;
  let user_id = Uuid::new_v4().to_string();
  sqlx::query("INSERT INTO users (id, username, password_hash) VALUES (?, ?, ?)")
    .bind(user_id)
    .bind(payload.username.trim())
    .bind(password_hash)
    .execute(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  Ok(Json(UserCreateResponse { username: payload.username.trim().to_string() }))
}

async fn user_change_password(
  State(state): State<AppState>,
  user: AuthUser,
  Json(payload): Json<ChangePasswordRequest>
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
  if user.username == state.config.admin_username {
    return Err((StatusCode::FORBIDDEN, Json(ErrorResponse::new("管理员密码需通过环境变量修改"))));
  }
  if payload.new_password.trim().len() < 6 {
    return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse::new("新密码长度至少 6 位"))));
  }

  let row: Option<(String,)> = sqlx::query_as("SELECT password_hash FROM users WHERE id = ?")
    .bind(&user.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;
  let Some((hash,)) = row else {
    return Err((StatusCode::NOT_FOUND, Json(ErrorResponse::new("用户不存在"))));
  };

  if !verify_password(&hash, payload.current_password.trim()) {
    return Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse::new("当前密码错误"))));
  }

  let next_hash = hash_password(payload.new_password.trim())
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Password hash failed"))))?;
  sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
    .bind(next_hash)
    .bind(&user.user_id)
    .execute(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  Ok(Json(json!({ "ok": true })))
}

async fn admin_reset_password(
  State(state): State<AppState>,
  user: AuthUser,
  Json(payload): Json<AdminResetPasswordRequest>
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
  if user.username != state.config.admin_username {
    return Err((StatusCode::FORBIDDEN, Json(ErrorResponse::new("仅管理员可重置密码"))));
  }
  let target = payload.username.trim();
  if target.is_empty() {
    return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse::new("用户名不能为空"))));
  }
  if target == state.config.admin_username {
    return Err((StatusCode::FORBIDDEN, Json(ErrorResponse::new("管理员密码需通过环境变量修改"))));
  }
  if payload.new_password.trim().len() < 6 {
    return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse::new("新密码长度至少 6 位"))));
  }

  let row: Option<(String,)> = sqlx::query_as("SELECT id FROM users WHERE username = ?")
    .bind(target)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;
  let Some((user_id,)) = row else {
    return Err((StatusCode::NOT_FOUND, Json(ErrorResponse::new("用户不存在"))));
  };

  let next_hash = hash_password(payload.new_password.trim())
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Password hash failed"))))?;
  sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
    .bind(next_hash)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  Ok(Json(json!({ "ok": true })))
}

async fn now_playing(State(state): State<AppState>, user: AuthUser) -> Result<Json<NowPlayingResponse>, (StatusCode, Json<ErrorResponse>)> {
  let row = sqlx::query(
    "SELECT t.id, t.title, t.artist, t.album, t.duration_seconds, t.quality, t.mood, t.source, t.source_id, t.cover_url, np.progress_percent \
     FROM now_playing np \
     JOIN tracks t ON np.track_id = t.id \
     WHERE np.user_id = ? AND t.user_id = ? \
     ORDER BY np.updated_at DESC LIMIT 1"
  )
    .bind(&user.user_id)
    .bind(&user.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  if let Some(row) = row {
    let duration: Option<i32> = row.try_get("duration_seconds").ok();
    let length = duration.map(format_duration).unwrap_or_else(|| "00:00".to_string());
    let quality: Option<String> = row.try_get("quality").ok();
    let mood: Option<String> = row.try_get("mood").ok();
    return Ok(Json(NowPlayingResponse {
      id: row.try_get("id").unwrap_or_default(),
      title: row.try_get("title").unwrap_or_default(),
      artist: row.try_get("artist").unwrap_or_default(),
      album: row.try_get("album").unwrap_or_default(),
      length,
      duration_seconds: duration,
      progress: row.try_get("progress_percent").unwrap_or(0),
      quality: quality.unwrap_or_else(|| "Hi-Res".to_string()),
      mood: mood.unwrap_or_else(|| "沉浸".to_string()),
      source: row.try_get("source").unwrap_or_else(|_| "local".to_string()),
      source_id: row.try_get("source_id").ok(),
      cover_url: row.try_get("cover_url").ok()
    }));
  }

  Ok(Json(NowPlayingResponse {
    id: "".to_string(),
    title: "未播放".to_string(),
    artist: "".to_string(),
    album: "".to_string(),
    length: "00:00".to_string(),
    duration_seconds: None,
    progress: 0,
    quality: "".to_string(),
    mood: "".to_string(),
    source: "".to_string(),
    source_id: None,
    cover_url: None
  }))
}

async fn queue_list(State(state): State<AppState>, user: AuthUser) -> Result<Json<Vec<QueueItem>>, (StatusCode, Json<ErrorResponse>)> {
  let rows = sqlx::query(
    "SELECT q.id AS queue_id, t.id, t.title, t.artist, t.duration_seconds, t.source, t.source_id, t.cover_url \
     FROM queue q \
     JOIN tracks t ON q.track_id = t.id \
     WHERE q.user_id = ? AND t.user_id = ? \
     ORDER BY q.position ASC"
  )
    .bind(&user.user_id)
    .bind(&user.user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  let items = rows.into_iter().map(|row| {
    let duration: Option<i32> = row.try_get("duration_seconds").ok();
    QueueItem {
      queue_id: row.try_get("queue_id").unwrap_or_default(),
      id: row.try_get("id").unwrap_or_default(),
      title: row.try_get("title").unwrap_or_default(),
      artist: row.try_get("artist").unwrap_or_default(),
      time: duration.map(format_duration).unwrap_or_else(|| "00:00".to_string()),
      duration_seconds: duration,
      source: row.try_get("source").unwrap_or_else(|_| "local".to_string()),
      source_id: row.try_get("source_id").ok(),
      cover_url: row.try_get("cover_url").ok()
    }
  }).collect();

  Ok(Json(items))
}

async fn playlists(State(state): State<AppState>, user: AuthUser) -> Result<Json<Vec<PlaylistItem>>, (StatusCode, Json<ErrorResponse>)> {
  let rows = sqlx::query(
    "SELECT p.id, p.name, p.description, p.mood, COUNT(pt.track_id) AS count \
     FROM playlists p \
     LEFT JOIN playlist_tracks pt ON p.id = pt.playlist_id \
     WHERE p.user_id = ? \
     GROUP BY p.id, p.name, p.description, p.mood \
     ORDER BY p.created_at DESC"
  )
    .bind(&user.user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  let items = rows.into_iter().map(|row| PlaylistItem {
    id: row.try_get("id").unwrap_or_default(),
    name: row.try_get("name").unwrap_or_default(),
    desc: row.try_get("description").unwrap_or_default(),
    count: row.try_get("count").unwrap_or(0),
    mood: row.try_get("mood").unwrap_or_default()
  }).collect();

  Ok(Json(items))
}

#[derive(Deserialize)]
struct LibraryQuery {
  limit: Option<i64>,
  offset: Option<i64>,
  tag: Option<String>
}

async fn library(State(state): State<AppState>, user: AuthUser, Query(query): Query<LibraryQuery>) -> Result<Json<Vec<LibraryItem>>, (StatusCode, Json<ErrorResponse>)> {
  let limit = query.limit.unwrap_or(20).clamp(1, 200);
  let offset = query.offset.unwrap_or(0).max(0);

  let rows = if let Some(tag) = query.tag {
    sqlx::query(
      "SELECT id, title, artist, album, duration_seconds, tag, source, source_id, cover_url FROM tracks WHERE user_id = ? AND tag = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
    )
      .bind(&user.user_id)
      .bind(tag)
      .bind(limit)
      .bind(offset)
      .fetch_all(&state.db)
      .await
  } else {
    sqlx::query(
      "SELECT id, title, artist, album, duration_seconds, tag, source, source_id, cover_url FROM tracks WHERE user_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
    )
      .bind(&user.user_id)
      .bind(limit)
      .bind(offset)
      .fetch_all(&state.db)
      .await
  }
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  let items = rows.into_iter().map(|row| {
    let duration: Option<i32> = row.try_get("duration_seconds").ok();
    LibraryItem {
      id: row.try_get("id").unwrap_or_default(),
      title: row.try_get("title").unwrap_or_default(),
      artist: row.try_get("artist").unwrap_or_default(),
      album: row.try_get("album").unwrap_or_default(),
      time: duration.map(format_duration).unwrap_or_else(|| "00:00".to_string()),
      duration_seconds: duration,
      tag: row.try_get("tag").unwrap_or_else(|_| "".to_string()),
      source: row.try_get("source").unwrap_or_else(|_| "local".to_string()),
      source_id: row.try_get("source_id").ok(),
      cover_url: row.try_get("cover_url").ok()
    }
  }).collect();

  Ok(Json(items))
}

async fn track_detail(State(state): State<AppState>, user: AuthUser, Path(id): Path<String>) -> Result<Json<LibraryItem>, (StatusCode, Json<ErrorResponse>)> {
  let row = sqlx::query(
    "SELECT id, title, artist, album, duration_seconds, tag, source, source_id, cover_url \
     FROM tracks WHERE id = ? AND user_id = ? LIMIT 1"
  )
    .bind(id)
    .bind(&user.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  let Some(row) = row else {
    return Err((StatusCode::NOT_FOUND, Json(ErrorResponse::new("Track not found"))));
  };

  let duration: Option<i32> = row.try_get("duration_seconds").ok();
  Ok(Json(LibraryItem {
    id: row.try_get("id").unwrap_or_default(),
    title: row.try_get("title").unwrap_or_default(),
    artist: row.try_get("artist").unwrap_or_default(),
    album: row.try_get("album").unwrap_or_default(),
    time: duration.map(format_duration).unwrap_or_else(|| "00:00".to_string()),
    duration_seconds: duration,
    tag: row.try_get("tag").unwrap_or_else(|_| "".to_string()),
    source: row.try_get("source").unwrap_or_else(|_| "local".to_string()),
    source_id: row.try_get("source_id").ok(),
    cover_url: row.try_get("cover_url").ok()
  }))
}

async fn playlist_detail(State(state): State<AppState>, user: AuthUser, Path(id): Path<String>) -> Result<Json<PlaylistDetail>, (StatusCode, Json<ErrorResponse>)> {
  let playlist_row = sqlx::query("SELECT id, name, description, mood FROM playlists WHERE id = ? AND user_id = ? LIMIT 1")
    .bind(&id)
    .bind(&user.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  let Some(playlist_row) = playlist_row else {
    return Err((StatusCode::NOT_FOUND, Json(ErrorResponse::new("Playlist not found"))));
  };

  let rows = sqlx::query(
    "SELECT t.id, t.title, t.artist, t.album, t.duration_seconds, t.tag, t.source, t.source_id, t.cover_url \
     FROM playlist_tracks pt \
     JOIN tracks t ON pt.track_id = t.id \
     WHERE pt.playlist_id = ? AND pt.user_id = ? AND t.user_id = ? \
     ORDER BY pt.position ASC"
  )
    .bind(&id)
    .bind(&user.user_id)
    .bind(&user.user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  let tracks = rows.into_iter().map(|row| {
    let duration: Option<i32> = row.try_get("duration_seconds").ok();
    LibraryItem {
      id: row.try_get("id").unwrap_or_default(),
      title: row.try_get("title").unwrap_or_default(),
      artist: row.try_get("artist").unwrap_or_default(),
      album: row.try_get("album").unwrap_or_default(),
      time: duration.map(format_duration).unwrap_or_else(|| "00:00".to_string()),
      duration_seconds: duration,
      tag: row.try_get("tag").unwrap_or_else(|_| "".to_string()),
      source: row.try_get("source").unwrap_or_else(|_| "local".to_string()),
      source_id: row.try_get("source_id").ok(),
      cover_url: row.try_get("cover_url").ok()
    }
  }).collect();

  Ok(Json(PlaylistDetail {
    id: playlist_row.try_get("id").unwrap_or_default(),
    name: playlist_row.try_get("name").unwrap_or_default(),
    desc: playlist_row.try_get("description").unwrap_or_default(),
    mood: playlist_row.try_get("mood").unwrap_or_default(),
    tracks
  }))
}

async fn queue_clear(State(state): State<AppState>, user: AuthUser) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
  sqlx::query("DELETE FROM queue WHERE user_id = ?")
    .bind(&user.user_id)
    .execute(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  Ok(Json(serde_json::json!({ "ok": true })))
}

async fn queue_add(State(state): State<AppState>, user: AuthUser, Json(payload): Json<QueueAddRequest>) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
  let (max_pos,): (Option<i32>,) = sqlx::query_as("SELECT MAX(position) FROM queue WHERE user_id = ?")
    .bind(&user.user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;
  let mut position = max_pos.unwrap_or(-1) + 1;

  for item in payload.items {
    let track_id = resolve_track_id(&state.db, &user.user_id, item.track_id, item.source, item.source_id, item.title, item.artist, item.album, item.cover_url).await?;
    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO queue (id, track_id, position, user_id) VALUES (?, ?, ?, ?)")
      .bind(id)
      .bind(track_id)
      .bind(position)
      .bind(&user.user_id)
      .execute(&state.db)
      .await
      .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;
    position += 1;
  }

  Ok(Json(serde_json::json!({ "ok": true })))
}

async fn queue_replace(State(state): State<AppState>, user: AuthUser, Json(payload): Json<QueueReplaceRequest>) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
  sqlx::query("DELETE FROM queue WHERE user_id = ?")
    .bind(&user.user_id)
    .execute(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  let mut position = 0;
  if let Some(playlist_id) = payload.playlist_id {
    let rows = sqlx::query("SELECT track_id FROM playlist_tracks WHERE playlist_id = ? AND user_id = ? ORDER BY position ASC")
      .bind(playlist_id)
      .bind(&user.user_id)
      .fetch_all(&state.db)
      .await
      .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;
    for row in rows {
      let track_id: String = row.try_get("track_id").unwrap_or_default();
      let id = Uuid::new_v4().to_string();
      sqlx::query("INSERT INTO queue (id, track_id, position, user_id) VALUES (?, ?, ?, ?)")
        .bind(id)
        .bind(track_id)
        .bind(position)
        .bind(&user.user_id)
        .execute(&state.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;
      position += 1;
    }
  } else if let Some(items) = payload.items {
    for item in items {
      let track_id = resolve_track_id(&state.db, &user.user_id, item.track_id, item.source, item.source_id, item.title, item.artist, item.album, item.cover_url).await?;
      let id = Uuid::new_v4().to_string();
      sqlx::query("INSERT INTO queue (id, track_id, position, user_id) VALUES (?, ?, ?, ?)")
        .bind(id)
        .bind(track_id)
        .bind(position)
        .bind(&user.user_id)
        .execute(&state.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;
      position += 1;
    }
  } else {
    return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse::new("Missing playlist_id or items"))));
  }

  Ok(Json(serde_json::json!({ "ok": true })))
}

async fn queue_reorder(State(state): State<AppState>, user: AuthUser, Json(payload): Json<QueueReorderRequest>) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
  if payload.order.is_empty() {
    return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse::new("Missing order"))));
  }

  let mut tx = state.db.begin()
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  for (idx, id) in payload.order.iter().enumerate() {
    sqlx::query("UPDATE queue SET position = ? WHERE id = ? AND user_id = ?")
      .bind(idx as i32)
      .bind(id)
      .bind(&user.user_id)
      .execute(&mut *tx)
      .await
      .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;
  }

  tx.commit()
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  Ok(Json(serde_json::json!({ "ok": true })))
}

async fn playlist_create(State(state): State<AppState>, user: AuthUser, Json(payload): Json<PlaylistCreateRequest>) -> Result<Json<PlaylistItem>, (StatusCode, Json<ErrorResponse>)> {
  if payload.name.trim().is_empty() {
    return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse::new("Playlist name is required"))));
  }

  let id = Uuid::new_v4().to_string();
  sqlx::query("INSERT INTO playlists (id, name, description, mood, user_id) VALUES (?, ?, ?, ?, ?)")
    .bind(&id)
    .bind(payload.name.trim())
    .bind(payload.desc.clone().unwrap_or_default())
    .bind(payload.mood.clone().unwrap_or_default())
    .bind(&user.user_id)
    .execute(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  Ok(Json(PlaylistItem {
    id,
    name: payload.name.trim().to_string(),
    desc: payload.desc.unwrap_or_default(),
    count: 0,
    mood: payload.mood.unwrap_or_default()
  }))
}

async fn playlist_update(State(state): State<AppState>, user: AuthUser, Path(id): Path<String>, Json(payload): Json<PlaylistUpdateRequest>) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
  let row = sqlx::query("SELECT id, name, description, mood FROM playlists WHERE id = ? AND user_id = ?")
    .bind(&id)
    .bind(&user.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;
  let Some(row) = row else {
    return Err((StatusCode::NOT_FOUND, Json(ErrorResponse::new("Playlist not found"))));
  };

  let name: String = row.try_get("name").unwrap_or_default();
  let desc: String = row.try_get("description").unwrap_or_default();
  let mood: String = row.try_get("mood").unwrap_or_default();

  let next_name = payload.name.unwrap_or(name);
  if next_name.trim().is_empty() {
    return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse::new("Playlist name is required"))));
  }

  let next_desc = payload.desc.unwrap_or(desc);
  let next_mood = payload.mood.unwrap_or(mood);

  sqlx::query("UPDATE playlists SET name = ?, description = ?, mood = ? WHERE id = ? AND user_id = ?")
    .bind(next_name.trim())
    .bind(next_desc)
    .bind(next_mood)
    .bind(&id)
    .bind(&user.user_id)
    .execute(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  Ok(Json(serde_json::json!({ "ok": true })))
}

async fn playlist_delete(State(state): State<AppState>, user: AuthUser, Path(id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
  let result = sqlx::query("DELETE FROM playlists WHERE id = ? AND user_id = ?")
    .bind(&id)
    .bind(&user.user_id)
    .execute(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  if result.rows_affected() == 0 {
    return Err((StatusCode::NOT_FOUND, Json(ErrorResponse::new("Playlist not found"))));
  }

  Ok(Json(serde_json::json!({ "ok": true })))
}

async fn playlist_add_track(
  State(state): State<AppState>,
  user: AuthUser,
  Path(id): Path<String>,
  Json(payload): Json<PlaylistAddTrackRequest>
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
  let track_id = resolve_track_id(&state.db, &user.user_id, payload.track_id, payload.source, payload.source_id, payload.title, payload.artist, payload.album, payload.cover_url).await?;
  let (max_pos,): (Option<i32>,) = sqlx::query_as("SELECT MAX(position) FROM playlist_tracks WHERE playlist_id = ? AND user_id = ?")
    .bind(&id)
    .bind(&user.user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;
  let position = max_pos.unwrap_or(-1) + 1;
  sqlx::query("INSERT INTO playlist_tracks (playlist_id, track_id, position, user_id) VALUES (?, ?, ?, ?)")
    .bind(&id)
    .bind(track_id)
    .bind(position)
    .bind(&user.user_id)
    .execute(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  Ok(Json(serde_json::json!({ "ok": true })))
}

async fn playlist_remove_track(
  State(state): State<AppState>,
  user: AuthUser,
  Path((id, track_id)): Path<(String, String)>
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
  let result = sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ? AND track_id = ? AND user_id = ?")
    .bind(&id)
    .bind(&track_id)
    .bind(&user.user_id)
    .execute(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  if result.rows_affected() == 0 {
    return Err((StatusCode::NOT_FOUND, Json(ErrorResponse::new("Track not found in playlist"))));
  }

  let rows = sqlx::query("SELECT track_id FROM playlist_tracks WHERE playlist_id = ? AND user_id = ? ORDER BY position ASC")
    .bind(&id)
    .bind(&user.user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  for (idx, row) in rows.iter().enumerate() {
    let item_id: String = row.try_get("track_id").unwrap_or_default();
    sqlx::query("UPDATE playlist_tracks SET position = ? WHERE playlist_id = ? AND track_id = ? AND user_id = ?")
      .bind(idx as i32)
      .bind(&id)
      .bind(item_id)
      .bind(&user.user_id)
      .execute(&state.db)
      .await
      .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;
  }

  Ok(Json(serde_json::json!({ "ok": true })))
}

async fn track_update(State(state): State<AppState>, user: AuthUser, Path(id): Path<String>, Json(payload): Json<TrackUpdateRequest>) -> Result<Json<LibraryItem>, (StatusCode, Json<ErrorResponse>)> {
  let row = sqlx::query(
    "SELECT id, title, artist, album, duration_seconds, tag, source, source_id, cover_url FROM tracks WHERE id = ? AND user_id = ? LIMIT 1"
  )
    .bind(&id)
    .bind(&user.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  let Some(row) = row else {
    return Err((StatusCode::NOT_FOUND, Json(ErrorResponse::new("Track not found"))));
  };

  let title: String = row.try_get("title").unwrap_or_default();
  let artist: String = row.try_get("artist").unwrap_or_default();
  let album: String = row.try_get("album").unwrap_or_default();
  let tag: String = row.try_get("tag").unwrap_or_default();
  let cover_url: Option<String> = row.try_get("cover_url").ok();
  let duration: Option<i32> = row.try_get("duration_seconds").ok();
  let source: String = row.try_get("source").unwrap_or_else(|_| "local".to_string());
  let source_id: Option<String> = row.try_get("source_id").ok();

  let next_title = payload.title.unwrap_or(title);
  let next_artist = payload.artist.unwrap_or(artist);
  let next_album = payload.album.unwrap_or(album);
  let next_tag = payload.tag.unwrap_or(tag);
  let next_cover = payload.cover_url.or(cover_url);
  let next_duration = payload.duration_seconds.or(duration);

  sqlx::query("UPDATE tracks SET title = ?, artist = ?, album = ?, tag = ?, cover_url = ?, duration_seconds = ? WHERE id = ? AND user_id = ?")
    .bind(&next_title)
    .bind(&next_artist)
    .bind(&next_album)
    .bind(&next_tag)
    .bind(&next_cover)
    .bind(next_duration)
    .bind(&id)
    .bind(&user.user_id)
    .execute(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  Ok(Json(LibraryItem {
    id,
    title: next_title,
    artist: next_artist,
    album: next_album,
    time: next_duration.map(format_duration).unwrap_or_else(|| "00:00".to_string()),
    duration_seconds: next_duration,
    tag: next_tag,
    source,
    source_id,
    cover_url: next_cover
  }))
}

async fn track_delete(State(state): State<AppState>, user: AuthUser, Path(id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
  let row = sqlx::query("SELECT file_path FROM tracks WHERE id = ? AND user_id = ? LIMIT 1")
    .bind(&id)
    .bind(&user.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  let Some(row) = row else {
    return Err((StatusCode::NOT_FOUND, Json(ErrorResponse::new("Track not found"))));
  };

  let file_path: Option<String> = row.try_get("file_path").ok();

  let result = sqlx::query("DELETE FROM tracks WHERE id = ? AND user_id = ?")
    .bind(&id)
    .bind(&user.user_id)
    .execute(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  if result.rows_affected() == 0 {
    return Err((StatusCode::NOT_FOUND, Json(ErrorResponse::new("Track not found"))));
  }

  if let Some(path) = file_path {
    let path = PathBuf::from(path);
    if path.exists() {
      let _ = fs::remove_file(path).await;
    }
  }

  Ok(Json(serde_json::json!({ "ok": true })))
}

async fn resolve_track_id(
  pool: &MySqlPool,
  user_id: &str,
  track_id: Option<String>,
  source: Option<String>,
  source_id: Option<String>,
  title: Option<String>,
  artist: Option<String>,
  album: Option<String>,
  cover_url: Option<String>
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
  if let Some(track_id) = track_id {
    return Ok(track_id);
  }

  let source = source.unwrap_or_else(|| "local".to_string());
  if source != "netease" {
    return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse::new("Missing track_id for local source"))));
  }

  let Some(source_id) = source_id else {
    return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse::new("Missing source_id"))));
  };

  let existing: Option<(String,)> = sqlx::query_as("SELECT id FROM tracks WHERE source = 'netease' AND source_id = ? AND user_id = ? LIMIT 1")
    .bind(&source_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  if let Some((id,)) = existing {
    return Ok(id);
  }

  let title = title.unwrap_or_else(|| "未命名".to_string());
  let artist = artist.unwrap_or_else(|| "未知".to_string());
  let album = album.unwrap_or_else(|| "".to_string());
  let track_id = Uuid::new_v4().to_string();
  sqlx::query("INSERT INTO tracks (id, title, artist, album, source, source_id, cover_url, tag, user_id) VALUES (?, ?, ?, ?, 'netease', ?, ?, '网易云', ?)")
    .bind(&track_id)
    .bind(title)
    .bind(artist)
    .bind(album)
    .bind(source_id)
    .bind(cover_url)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  Ok(track_id)
}

async fn stream_track(State(state): State<AppState>, user: AuthUser, Path(id): Path<String>, headers: HeaderMap) -> Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
  let row = sqlx::query("SELECT file_path FROM tracks WHERE id = ? AND user_id = ? LIMIT 1")
    .bind(&id)
    .bind(&user.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  let Some(row) = row else {
    return Err((StatusCode::NOT_FOUND, Json(ErrorResponse::new("Track not found"))));
  };
  let file_path: Option<String> = row.try_get("file_path").ok();
  let Some(file_path) = file_path else {
    return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse::new("Track has no file"))));
  };

  let path = PathBuf::from(file_path);
  let metadata = fs::metadata(&path)
    .await
    .map_err(|_| (StatusCode::NOT_FOUND, Json(ErrorResponse::new("File not found"))))?;
  let file_size = metadata.len();

  let content_type = match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
    "mp3" => "audio/mpeg",
    "flac" => "audio/flac",
    "m4a" => "audio/mp4",
    "wav" => "audio/wav",
    "ogg" => "audio/ogg",
    "aac" => "audio/aac",
    _ => "application/octet-stream"
  };

  if let Some(range_header) = headers.get(header::RANGE).and_then(|v| v.to_str().ok()) {
    if let Some(range) = range_header.strip_prefix("bytes=") {
      let parts: Vec<&str> = range.split('-').collect();
      if let Some(start_str) = parts.get(0) {
        let start: u64 = start_str.parse().unwrap_or(0);
        let end: u64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(file_size - 1);
        let end = end.min(file_size - 1);
        let len = end - start + 1;
        let mut file = fs::File::open(&path)
          .await
          .map_err(|_| (StatusCode::NOT_FOUND, Json(ErrorResponse::new("File not found"))))?;
        file.seek(SeekFrom::Start(start)).await
          .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Seek failed"))))?;
        let stream = ReaderStream::new(file.take(len));
        let mut response = axum::response::Response::new(Body::from_stream(stream));
        *response.status_mut() = StatusCode::PARTIAL_CONTENT;
        let headers = response.headers_mut();
        headers.insert(header::CONTENT_TYPE, HeaderValue::from_str(content_type).unwrap());
        headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
        headers.insert(header::CONTENT_LENGTH, HeaderValue::from_str(&len.to_string()).unwrap());
        headers.insert(header::CONTENT_RANGE, HeaderValue::from_str(&format!("bytes {}-{}/{}", start, end, file_size)).unwrap());
        return Ok(response);
      }
    }
  }

  let file = fs::File::open(&path)
    .await
    .map_err(|_| (StatusCode::NOT_FOUND, Json(ErrorResponse::new("File not found"))))?;
  let stream = ReaderStream::new(file);
  let mut response = axum::response::Response::new(Body::from_stream(stream));
  let headers = response.headers_mut();
  headers.insert(header::CONTENT_TYPE, HeaderValue::from_str(content_type).unwrap());
  headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
  headers.insert(header::CONTENT_LENGTH, HeaderValue::from_str(&file_size.to_string()).unwrap());
  Ok(response)
}

async fn local_track_url(State(state): State<AppState>, user: AuthUser, Path(id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
  let row = sqlx::query("SELECT file_path FROM tracks WHERE id = ? AND user_id = ? LIMIT 1")
    .bind(&id)
    .bind(&user.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  let Some(row) = row else {
    return Err((StatusCode::NOT_FOUND, Json(ErrorResponse::new("Track not found"))));
  };
  let file_path: Option<String> = row.try_get("file_path").ok();
  let Some(file_path) = file_path else {
    return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse::new("Track has no file"))));
  };
  let path = PathBuf::from(file_path);
  let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
  if file_name.is_empty() {
    return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse::new("Invalid file path"))));
  }

  if let Some(base) = &state.config.public_media_base {
    let base = base.trim_end_matches('/');
    let url = format!("{}/{}", base, file_name);
    return Ok(Json(json!({ "url": url, "public": true })));
  }

  Ok(Json(json!({ "url": format!("/api/stream/{}", id), "public": false })))
}

async fn local_lyric(State(state): State<AppState>, user: AuthUser, Path(id): Path<String>) -> Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
  let exists: Option<(String,)> = sqlx::query_as("SELECT id FROM tracks WHERE id = ? AND user_id = ? LIMIT 1")
    .bind(&id)
    .bind(&user.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;
  if exists.is_none() {
    return Err((StatusCode::NOT_FOUND, Json(ErrorResponse::new("Track not found"))));
  }
  let lyric_path = state.config.music_storage_dir.join("lyrics").join(format!("{}.lrc", id));
  if !lyric_path.exists() {
    return Err((StatusCode::NOT_FOUND, Json(ErrorResponse::new("Lyric not found"))));
  }
  let content = fs::read_to_string(&lyric_path)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Lyric read failed"))))?;
  let mut response = axum::response::Response::new(Body::from(content));
  response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("text/plain; charset=utf-8"));
  Ok(response)
}

async fn devices(State(state): State<AppState>, user: AuthUser) -> Result<Json<Vec<DeviceItem>>, (StatusCode, Json<ErrorResponse>)> {
  let rows = sqlx::query(
    "SELECT id, name, status, description, \
      DATE_FORMAT(last_seen_at, '%Y-%m-%d %H:%i') AS last_seen_at, \
      DATE_FORMAT(last_sync_at, '%Y-%m-%d %H:%i') AS last_sync_at, \
      cache_size_mb, synced_tracks, playing_title, playing_artist, playback_progress \
     FROM devices WHERE user_id = ? ORDER BY name ASC"
  )
    .bind(&user.user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  let items = rows.into_iter().map(|row| DeviceItem {
    id: row.try_get("id").unwrap_or_default(),
    name: row.try_get("name").unwrap_or_default(),
    status: row.try_get("status").unwrap_or_default(),
    desc: row.try_get("description").unwrap_or_default(),
    last_seen_at: row.try_get("last_seen_at").ok(),
    last_sync_at: row.try_get("last_sync_at").ok(),
    cache_size_mb: row.try_get("cache_size_mb").ok(),
    synced_tracks: row.try_get("synced_tracks").ok(),
    playing_title: row.try_get("playing_title").ok(),
    playing_artist: row.try_get("playing_artist").ok(),
    playback_progress: row.try_get("playback_progress").ok()
  }).collect();

  Ok(Json(items))
}

async fn device_create(
  State(state): State<AppState>,
  user: AuthUser,
  Json(payload): Json<DeviceCreateRequest>
) -> Result<Json<DeviceCreateResponse>, (StatusCode, Json<ErrorResponse>)> {
  if payload.name.trim().is_empty() {
    return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse::new("Device name is required"))));
  }
  let id = Uuid::new_v4().to_string();
  let token = Uuid::new_v4().to_string();
  sqlx::query("INSERT INTO devices (id, name, status, description, device_token, user_id) VALUES (?, ?, ?, ?, ?, ?)")
    .bind(&id)
    .bind(payload.name.trim())
    .bind("待同步")
    .bind(payload.desc.unwrap_or_default())
    .bind(&token)
    .bind(&user.user_id)
    .execute(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  Ok(Json(DeviceCreateResponse { id, token }))
}

async fn device_sync(
  State(state): State<AppState>,
  user: AuthUser,
  Path(id): Path<String>
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
  let result = sqlx::query("UPDATE devices SET status = '同步中', last_sync_at = ? WHERE id = ? AND user_id = ?")
    .bind(Utc::now().naive_utc())
    .bind(&id)
    .bind(&user.user_id)
    .execute(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;
  if result.rows_affected() == 0 {
    return Err((StatusCode::NOT_FOUND, Json(ErrorResponse::new("Device not found"))));
  }
  Ok(Json(serde_json::json!({ "ok": true })))
}

async fn device_update(
  State(state): State<AppState>,
  user: AuthUser,
  Path(id): Path<String>,
  Json(payload): Json<DeviceUpdateRequest>
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
  let row = sqlx::query(
    "SELECT status, description, cache_size_mb, synced_tracks, playing_title, playing_artist, playback_progress FROM devices WHERE id = ? AND user_id = ?"
  )
    .bind(&id)
    .bind(&user.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;
  let Some(row) = row else {
    return Err((StatusCode::NOT_FOUND, Json(ErrorResponse::new("Device not found"))));
  };

  let status: String = row.try_get("status").unwrap_or_default();
  let desc: String = row.try_get("description").unwrap_or_default();
  let cache_size_mb: Option<i32> = row.try_get("cache_size_mb").ok();
  let synced_tracks: Option<i32> = row.try_get("synced_tracks").ok();
  let playing_title: Option<String> = row.try_get("playing_title").ok();
  let playing_artist: Option<String> = row.try_get("playing_artist").ok();
  let playback_progress: Option<i32> = row.try_get("playback_progress").ok();

  let next_status = payload.status.unwrap_or(status);
  let next_desc = payload.desc.unwrap_or(desc);
  let next_cache = payload.cache_size_mb.or(cache_size_mb);
  let next_synced = payload.synced_tracks.or(synced_tracks);
  let next_title = payload.playing_title.or(playing_title);
  let next_artist = payload.playing_artist.or(playing_artist);
  let next_progress = payload.playback_progress.or(playback_progress);

  sqlx::query(
    "UPDATE devices SET status = ?, description = ?, cache_size_mb = ?, synced_tracks = ?, playing_title = ?, playing_artist = ?, playback_progress = ?, last_seen_at = ? WHERE id = ? AND user_id = ?"
  )
    .bind(next_status)
    .bind(next_desc)
    .bind(next_cache)
    .bind(next_synced)
    .bind(next_title)
    .bind(next_artist)
    .bind(next_progress)
    .bind(Utc::now().naive_utc())
    .bind(&id)
    .bind(&user.user_id)
    .execute(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  Ok(Json(serde_json::json!({ "ok": true })))
}

async fn device_delete(
  State(state): State<AppState>,
  user: AuthUser,
  Path(id): Path<String>
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
  let result = sqlx::query("DELETE FROM devices WHERE id = ? AND user_id = ?")
    .bind(&id)
    .bind(&user.user_id)
    .execute(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;
  if result.rows_affected() == 0 {
    return Err((StatusCode::NOT_FOUND, Json(ErrorResponse::new("Device not found"))));
  }
  Ok(Json(serde_json::json!({ "ok": true })))
}

async fn device_report(
  State(state): State<AppState>,
  Path(id): Path<String>,
  headers: HeaderMap,
  Json(payload): Json<DeviceReportRequest>
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
  let token = headers.get("x-device-token").and_then(|v| v.to_str().ok());
  let Some(token) = token else {
    return Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse::new("Missing device token"))));
  };

  let row = sqlx::query(
    "SELECT device_token, status, cache_size_mb, synced_tracks, playing_title, playing_artist, playback_progress, last_sync_at \
     FROM devices WHERE id = ?"
  )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  let Some(row) = row else {
    return Err((StatusCode::NOT_FOUND, Json(ErrorResponse::new("Device not found"))));
  };

  let stored: Option<String> = row.try_get("device_token").ok();
  if stored.as_deref() != Some(token) {
    return Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse::new("Invalid device token"))));
  }

  let status: String = row.try_get("status").unwrap_or_else(|_| "在线".to_string());
  let cache_size_mb: Option<i32> = row.try_get("cache_size_mb").ok();
  let synced_tracks: Option<i32> = row.try_get("synced_tracks").ok();
  let playing_title: Option<String> = row.try_get("playing_title").ok();
  let playing_artist: Option<String> = row.try_get("playing_artist").ok();
  let playback_progress: Option<i32> = row.try_get("playback_progress").ok();
  let last_sync_at: Option<chrono::NaiveDateTime> = row.try_get("last_sync_at").ok();

  let next_status = payload.status.unwrap_or(status);
  let next_cache = payload.cache_size_mb.or(cache_size_mb);
  let next_synced = payload.synced_tracks.or(synced_tracks);
  let next_title = payload.playing_title.or(playing_title);
  let next_artist = payload.playing_artist.or(playing_artist);
  let next_progress = payload.playback_progress.or(playback_progress);
  let next_sync_at = if payload.synced.unwrap_or(false) { Some(Utc::now().naive_utc()) } else { last_sync_at };

  sqlx::query(
    "UPDATE devices SET status = ?, cache_size_mb = ?, synced_tracks = ?, playing_title = ?, playing_artist = ?, playback_progress = ?, last_seen_at = ?, last_sync_at = ? WHERE id = ?"
  )
    .bind(next_status)
    .bind(next_cache)
    .bind(next_synced)
    .bind(next_title)
    .bind(next_artist)
    .bind(next_progress)
    .bind(Utc::now().naive_utc())
    .bind(next_sync_at)
    .bind(&id)
    .execute(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  Ok(Json(serde_json::json!({ "ok": true })))
}

async fn stats(State(state): State<AppState>, user: AuthUser) -> Result<Json<Vec<StatItem>>, (StatusCode, Json<ErrorResponse>)> {
  let track_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tracks WHERE user_id = ?")
    .bind(&user.user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  let offline_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tracks WHERE user_id = ? AND file_path IS NOT NULL")
    .bind(&user.user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  let listening_seconds = match sqlx::query_as::<_, (i64,)>(
    "SELECT CAST(COALESCE(SUM(COALESCE(e.duration_seconds, t.duration_seconds)), 0) AS SIGNED) \
     FROM play_events e \
     LEFT JOIN tracks t ON e.track_id = t.id \
     WHERE e.user_id = ? AND DATE(e.played_at) = CURDATE()"
  )
    .bind(&user.user_id)
    .fetch_one(&state.db)
    .await
  {
    Ok((val,)) => val,
    Err(err) => {
      tracing::warn!(error = %err, "play_events missing or query failed");
      0
    }
  };

  let today_listened_count = match sqlx::query_as::<_, (i64,)>(
    "SELECT COUNT(*) FROM play_events WHERE user_id = ? AND DATE(played_at) = CURDATE()"
  )
    .bind(&user.user_id)
    .fetch_one(&state.db)
    .await
  {
    Ok((val,)) => val,
    Err(err) => {
      tracing::warn!(error = %err, "play_events missing or query failed");
      0
    }
  };

  let hours = listening_seconds / 3600;
  let minutes = (listening_seconds % 3600) / 60;
  let listening_time = if hours > 0 {
    format!("{}h {}m", hours, minutes)
  } else {
    format!("{}m", minutes)
  };
  let today_listened = format!("{} 首", today_listened_count);

  let items = vec![
    StatItem { label: "收藏歌曲".to_string(), value: format!("{}", track_count.0) },
    StatItem { label: "离线曲目".to_string(), value: format!("{}", offline_count.0) },
    StatItem { label: "当前播放时长".to_string(), value: listening_time },
    StatItem { label: "今日收听".to_string(), value: today_listened }
  ];

  Ok(Json(items))
}

async fn playback_event(State(state): State<AppState>, user: AuthUser, Json(payload): Json<PlaybackEventRequest>) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
  let id = Uuid::new_v4().to_string();
  let source = payload.source.unwrap_or_else(|| "local".to_string());
  if let Err(err) = sqlx::query(
    "INSERT INTO play_events (id, track_id, source, source_id, title, artist, duration_seconds, user_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
  )
    .bind(id)
    .bind(payload.track_id)
    .bind(source)
    .bind(payload.source_id)
    .bind(payload.title)
    .bind(payload.artist)
    .bind(payload.duration_seconds)
    .bind(&user.user_id)
    .execute(&state.db)
    .await
  {
    tracing::warn!(error = %err, "play_events insert failed");
    return Ok(Json(json!({ "ok": false })));
  }

  Ok(Json(json!({ "ok": true })))
}

async fn playback_trend(State(state): State<AppState>, user: AuthUser, Query(query): Query<PlaybackTrendQuery>) -> Result<Json<PlaybackTrendResponse>, (StatusCode, Json<ErrorResponse>)> {
  let days = query.days.unwrap_or(9).clamp(3, 30);
  let today = Utc::now().date_naive();
  let start = today - ChronoDuration::days(days - 1);

  let rows = sqlx::query("SELECT DATE(played_at) AS day, COUNT(*) AS count FROM play_events WHERE user_id = ? AND played_at >= ? GROUP BY day ORDER BY day ASC")
    .bind(&user.user_id)
    .bind(start)
    .fetch_all(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  let mut map = HashMap::new();
  for row in rows {
    if let Ok(day) = row.try_get::<chrono::NaiveDate, _>("day") {
      let count: i64 = row.try_get::<i64, _>("count").unwrap_or(0);
      map.insert(day, count);
    }
  }

  let mut labels = Vec::new();
  let mut values = Vec::new();
  for offset in 0..days {
    let day = start + ChronoDuration::days(offset);
    labels.push(day.format("%m-%d").to_string());
    values.push(*map.get(&day).unwrap_or(&0));
  }

  Ok(Json(PlaybackTrendResponse {
    label: format!("最近 {} 天", days),
    labels,
    values
  }))
}

async fn upload_music(State(state): State<AppState>, user: AuthUser, mut multipart: Multipart) -> Result<Json<LibraryItem>, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!(user_id = %user.user_id, "upload_music start");
  let mut file_bytes = None;
  let mut filename = None;
  let mut title = None;
  let mut artist = None;
  let mut album = None;
  let mut tag = None;
  let mut lyric_bytes = None;
  let mut cover_bytes = None;
  let mut cover_filename = None;

  while let Some(field) = multipart.next_field().await.map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse::new("Invalid multipart"))))? {
    let name = field.name().unwrap_or("").to_string();
    if name == "file" {
      filename = field.file_name().map(|f| f.to_string());
      let bytes = field.bytes().await.map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse::new("Invalid file"))))?;
      tracing::info!(bytes = bytes.len(), "upload_music file received");
      file_bytes = Some(bytes);
    } else if name == "cover" || name == "image" || name == "art" {
      cover_filename = field.file_name().map(|f| f.to_string());
      let bytes = field.bytes().await.map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse::new("Invalid cover file"))))?;
      cover_bytes = Some(bytes);
    } else if name == "lyric" || name == "lrc" {
      let bytes = field.bytes().await.map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse::new("Invalid lyric file"))))?;
      lyric_bytes = Some(bytes);
    } else {
      let value = field.text().await.unwrap_or_default();
      match name.as_str() {
        "title" => title = Some(value),
        "artist" => artist = Some(value),
        "album" => album = Some(value),
        "tag" => tag = Some(value),
        _ => {}
      }
    }
  }

  let Some(bytes) = file_bytes else {
    return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse::new("Missing file"))));
  };

  let ext = filename.as_deref().and_then(|n| StdPath::new(n).extension().and_then(|e| e.to_str())).unwrap_or("bin");
  let file_id = Uuid::new_v4();
  let storage_path = state.config.music_storage_dir.join(format!("{}.{}", file_id, ext));

  let mut file = fs::File::create(&storage_path).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("File save failed"))))?;
  file.write_all(&bytes).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("File save failed"))))?;

  let track_id = Uuid::new_v4().to_string();

  let mut duration_seconds: Option<i32> = None;
  let mut embedded_lyrics: Option<String> = None;
  let mut cover_url: Option<String> = None;
  if let Ok(tagged) = Probe::open(&storage_path).and_then(|p| p.read()) {
    if let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) {
      if title.is_none() {
        title = tag.title().map(|v| v.to_string());
      }
      if artist.is_none() {
        artist = tag.artist().map(|v| v.to_string());
      }
      if album.is_none() {
        album = tag.album().map(|v| v.to_string());
      }
      if lyric_bytes.is_none() {
        embedded_lyrics = tag.get_string(&ItemKey::Lyrics).map(|v| v.to_string());
      }
    }
    let duration = tagged.properties().duration().as_secs();
    if duration > 0 {
      duration_seconds = Some(duration as i32);
    }
  }

  if let Some(cover_bytes) = cover_bytes {
    let cover_dir = state.config.music_storage_dir.join("covers");
    ensure_storage_dir(&cover_dir).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Cover save failed"))))?;
    let ext = cover_filename
      .as_deref()
      .and_then(|n| StdPath::new(n).extension().and_then(|e| e.to_str()))
      .unwrap_or("jpg");
    let cover_path = cover_dir.join(format!("{}.{}", track_id, ext));
    let mut cover_file = fs::File::create(&cover_path)
      .await
      .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Cover save failed"))))?;
    cover_file.write_all(&cover_bytes)
      .await
      .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Cover save failed"))))?;
    cover_url = Some(format!("/api/cover/{}.{}", track_id, ext));
  }

  if let Some(lyric_bytes) = lyric_bytes {
    let lyric_dir = state.config.music_storage_dir.join("lyrics");
    ensure_storage_dir(&lyric_dir).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Lyric save failed"))))?;
    let lyric_path = lyric_dir.join(format!("{}.lrc", track_id));
    let mut lyric_file = fs::File::create(&lyric_path).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Lyric save failed"))))?;
    lyric_file.write_all(&lyric_bytes).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Lyric save failed"))))?;
  } else if let Some(text) = embedded_lyrics {
    let lyric_dir = state.config.music_storage_dir.join("lyrics");
    ensure_storage_dir(&lyric_dir).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Lyric save failed"))))?;
    let lyric_path = lyric_dir.join(format!("{}.lrc", track_id));
    let mut lyric_file = fs::File::create(&lyric_path).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Lyric save failed"))))?;
    lyric_file.write_all(text.as_bytes()).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Lyric save failed"))))?;
  }

  let title_value = title.or_else(|| filename.clone()).unwrap_or_else(|| "未命名".to_string());
  let artist_value = artist.unwrap_or_else(|| "未知".to_string());
  let album_value = album.unwrap_or_else(|| "".to_string());
  let tag_value = tag.unwrap_or_else(|| "离线".to_string());

  if cover_url.is_none() {
    if let Some(found) = fetch_netease_cover_url(&state, &title_value, &artist_value).await {
      cover_url = Some(found);
    }
  }

  sqlx::query("INSERT INTO tracks (id, title, artist, album, duration_seconds, source, file_path, tag, cover_url, user_id) VALUES (?, ?, ?, ?, ?, 'local', ?, ?, ?, ?)")
    .bind(&track_id)
    .bind(&title_value)
    .bind(&artist_value)
    .bind(&album_value)
    .bind(duration_seconds)
    .bind(storage_path.to_string_lossy().to_string())
    .bind(&tag_value)
    .bind(&cover_url)
    .bind(&user.user_id)
    .execute(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("Database error"))))?;

  Ok(Json(LibraryItem {
    id: track_id,
    title: title_value,
    artist: artist_value,
    album: album_value,
    time: "00:00".to_string(),
    duration_seconds,
    tag: tag_value,
    source: "local".to_string(),
    source_id: None,
    cover_url
  }))
}

async fn netease_search(State(state): State<AppState>, Query(params): Query<HashMap<String, String>>) -> Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
  netease_proxy(&state, "/cloudsearch", params).await
}

async fn netease_login_cellphone(State(state): State<AppState>, Query(params): Query<HashMap<String, String>>) -> Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
  netease_proxy(&state, "/login/cellphone", params).await
}

async fn netease_login_email(State(state): State<AppState>, Query(params): Query<HashMap<String, String>>) -> Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
  netease_proxy(&state, "/login", params).await
}

async fn netease_login_status(State(state): State<AppState>, Query(params): Query<HashMap<String, String>>) -> Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
  netease_proxy(&state, "/login/status", params).await
}

async fn netease_login_status_post(
  State(state): State<AppState>,
  Query(params): Query<HashMap<String, String>>,
  Json(body): Json<serde_json::Value>
) -> Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
  netease_proxy_post(&state, "/login/status", params, body).await
}

async fn netease_logout(State(state): State<AppState>, Query(params): Query<HashMap<String, String>>) -> Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
  netease_proxy(&state, "/logout", params).await
}

async fn netease_user_account(State(state): State<AppState>, Query(params): Query<HashMap<String, String>>) -> Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
  netease_proxy(&state, "/user/account", params).await
}

async fn netease_captcha_sent(State(state): State<AppState>, Query(params): Query<HashMap<String, String>>) -> Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
  netease_proxy(&state, "/captcha/sent", params).await
}

async fn netease_captcha_verify(State(state): State<AppState>, Query(params): Query<HashMap<String, String>>) -> Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
  netease_proxy(&state, "/captcha/verify", params).await
}

async fn netease_login_qr_key(State(state): State<AppState>, Query(params): Query<HashMap<String, String>>) -> Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
  netease_proxy(&state, "/login/qr/key", params).await
}

async fn netease_login_qr_create(State(state): State<AppState>, Query(params): Query<HashMap<String, String>>) -> Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
  netease_proxy(&state, "/login/qr/create", params).await
}

async fn netease_login_qr_check(State(state): State<AppState>, Query(params): Query<HashMap<String, String>>) -> Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
  netease_proxy(&state, "/login/qr/check", params).await
}

async fn netease_song_detail(State(state): State<AppState>, Query(params): Query<HashMap<String, String>>) -> Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
  netease_proxy(&state, "/song/detail", params).await
}

async fn netease_song_url(State(state): State<AppState>, Query(params): Query<HashMap<String, String>>) -> Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
  netease_proxy(&state, "/song/url", params).await
}

async fn netease_song_url_v1(State(state): State<AppState>, Query(params): Query<HashMap<String, String>>) -> Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
  netease_proxy(&state, "/song/url/v1", params).await
}

async fn netease_lyric(State(state): State<AppState>, Query(params): Query<HashMap<String, String>>) -> Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
  netease_proxy(&state, "/lyric", params).await
}

async fn netease_lyric_new(State(state): State<AppState>, Query(params): Query<HashMap<String, String>>) -> Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
  netease_proxy(&state, "/lyric/new", params).await
}

async fn netease_playlist_detail(State(state): State<AppState>, Query(params): Query<HashMap<String, String>>) -> Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
  netease_proxy(&state, "/playlist/detail", params).await
}

async fn netease_playlist_track_all(State(state): State<AppState>, Query(params): Query<HashMap<String, String>>) -> Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
  netease_proxy(&state, "/playlist/track/all", params).await
}

async fn netease_stream(
  State(state): State<AppState>,
  headers: HeaderMap,
  Query(params): Query<HashMap<String, String>>
) -> Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
  let id = params.get("id").cloned().ok_or_else(|| (StatusCode::BAD_REQUEST, Json(ErrorResponse::new("Missing id"))))?;
  let level = params.get("level").cloned().unwrap_or_else(|| "exhigh".to_string());
  let unblock = params.get("unblock").cloned().unwrap_or_else(|| "true".to_string());
  let cookie = params.get("cookie").cloned();

  let mut query = HashMap::new();
  query.insert("id".to_string(), id);
  query.insert("level".to_string(), level);
  query.insert("unblock".to_string(), unblock);
  if let Some(cookie) = cookie {
    query.insert("cookie".to_string(), cookie);
  }

  let url_resp = match netease_proxy(&state, "/song/url/v1", query.clone()).await {
    Ok(resp) => resp,
    Err(_) => {
      tracing::warn!("song/url/v1 failed, retrying with standard level");
      let mut retry = query;
      retry.insert("level".to_string(), "standard".to_string());
      netease_proxy(&state, "/song/url/v1", retry).await?
    }
  };
  let status = url_resp.status();
  let body = axum::body::to_bytes(url_resp.into_body(), usize::MAX)
    .await
    .map_err(|_| (StatusCode::BAD_GATEWAY, Json(ErrorResponse::new("Upstream read failed"))))?;
  if !status.is_success() {
    return Ok((status, Body::from(body)).into_response());
  }
  let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or_else(|_| json!({}));
  let stream_url = json.get("data")
    .and_then(|v| v.get(0))
    .and_then(|v| v.get("url"))
    .and_then(|v| v.as_str())
    .unwrap_or("");
  if stream_url.is_empty() {
    return Err((StatusCode::BAD_GATEWAY, Json(ErrorResponse::new("No stream url"))));
  }

  let mut req = state.http.get(stream_url)
    .header(header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
    .header(header::ACCEPT_ENCODING, "identity");
  if let Some(range) = headers.get(header::RANGE) {
    req = req.header(header::RANGE, range);
  }
  let upstream = match req.send().await {
    Ok(resp) => resp,
    Err(err) => {
      tracing::warn!(error = %err, "Stream fetch failed, retrying without range");
      let retry = state.http.get(stream_url)
        .header(header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
        .header(header::ACCEPT_ENCODING, "identity")
        .send()
        .await
        .map_err(|_| (StatusCode::BAD_GATEWAY, Json(ErrorResponse::new("Stream fetch failed"))))?;
      retry
    }
  };

  let upstream_status = upstream.status();
  let upstream_headers = upstream.headers().clone();
  let mut response = axum::response::Response::new(Body::from_stream(upstream.bytes_stream()));
  *response.status_mut() = upstream_status;
  let headers_mut = response.headers_mut();
  if let Some(val) = upstream_headers.get(header::CONTENT_TYPE) {
    headers_mut.insert(header::CONTENT_TYPE, val.clone());
  }
  headers_mut.remove(header::CONTENT_LENGTH);
  if let Some(val) = upstream_headers.get(header::CONTENT_RANGE) {
    headers_mut.insert(header::CONTENT_RANGE, val.clone());
  }
  headers_mut.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
  Ok(response)
}

async fn netease_proxy(state: &AppState, path: &str, params: HashMap<String, String>) -> Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
  let url = format!("{}{}", state.config.netease_base_url.trim_end_matches('/'), path);
  let build_url = |base: &str, map: &HashMap<String, String>| -> Result<reqwest::Url, (StatusCode, Json<ErrorResponse>)> {
    let mut full_url = reqwest::Url::parse(base)
      .map_err(|_| (StatusCode::BAD_GATEWAY, Json(ErrorResponse::new("Invalid upstream URL"))))?;
    {
      let mut pairs = full_url.query_pairs_mut();
      for (k, v) in map {
        pairs.append_pair(k, v);
      }
    }
    Ok(full_url)
  };

  async fn fetch_upstream(
    client: &reqwest::Client,
    full_url: reqwest::Url,
    path: &str
  ) -> Result<(StatusCode, Bytes), (StatusCode, Json<ErrorResponse>)> {
    tracing::info!(upstream = %full_url, "Netease request");
    let resp = client.get(full_url)
      .header(header::ACCEPT_ENCODING, "identity")
      .send()
      .await
      .map_err(|err| {
        tracing::error!(error = %err, %path, "Netease request failed");
        (StatusCode::BAD_GATEWAY, Json(ErrorResponse::new(&format!("Netease API error: {}", err))))
      })?;

    let status = resp.status();
    let body = resp.bytes().await
      .map_err(|err| {
        tracing::error!(error = %err, %path, "Netease response read failed");
        (StatusCode::BAD_GATEWAY, Json(ErrorResponse::new(&format!("Netease API error: {}", err))))
      })?;
    Ok((status, body))
  }

  let full_url = build_url(&url, &params)?;
  let (status, body) = fetch_upstream(&state.http, full_url, path).await?;

  if path == "/login/qr/check" && status == StatusCode::BAD_GATEWAY && !params.contains_key("noCookie") {
    let mut retry_params = params.clone();
    retry_params.insert("noCookie".to_string(), "true".to_string());
    let retry_url = build_url(&url, &retry_params)?;
    tracing::warn!("Netease qr/check returned 502, retrying with noCookie=true");
    let (retry_status, retry_body) = fetch_upstream(&state.http, retry_url, path).await?;
    let retry_json = serde_json::from_slice(&retry_body)
      .unwrap_or_else(|_| json!({ "raw": String::from_utf8_lossy(&retry_body).to_string() }));
    if !retry_status.is_success() {
      tracing::warn!(%retry_status, %path, body = %String::from_utf8_lossy(&retry_body), "Netease upstream returned error");
    }
    return Ok((retry_status, Json(retry_json)).into_response());
  }

  let json = serde_json::from_slice(&body).unwrap_or_else(|_| json!({ "raw": String::from_utf8_lossy(&body).to_string() }));
  if !status.is_success() {
    tracing::warn!(%status, %path, body = %String::from_utf8_lossy(&body), "Netease upstream returned error");
  }
  Ok((status, Json(json)).into_response())
}

async fn netease_proxy_post(
  state: &AppState,
  path: &str,
  params: HashMap<String, String>,
  body: serde_json::Value
) -> Result<axum::response::Response, (StatusCode, Json<ErrorResponse>)> {
  let url = format!("{}{}", state.config.netease_base_url.trim_end_matches('/'), path);
  let mut full_url = reqwest::Url::parse(&url)
    .map_err(|_| (StatusCode::BAD_GATEWAY, Json(ErrorResponse::new("Invalid upstream URL"))))?;
  {
    let mut pairs = full_url.query_pairs_mut();
    for (k, v) in &params {
      pairs.append_pair(k, v);
    }
  }

  tracing::info!(upstream = %full_url, "Netease request (POST)");

  let resp = state.http.post(full_url)
    .header(header::ACCEPT_ENCODING, "identity")
    .json(&body)
    .send()
    .await
    .map_err(|err| {
      tracing::error!(error = %err, %path, "Netease request failed");
      (StatusCode::BAD_GATEWAY, Json(ErrorResponse::new(&format!("Netease API error: {}", err))))
    })?;

  let status = resp.status();
  let text = resp.text().await
    .map_err(|err| {
      tracing::error!(error = %err, %path, "Netease response read failed");
      (StatusCode::BAD_GATEWAY, Json(ErrorResponse::new(&format!("Netease API error: {}", err))))
    })?;
  let json = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));

  if !status.is_success() {
    tracing::warn!(%status, %path, body = %text, "Netease upstream returned error");
  }

  Ok((status, Json(json)).into_response())
}

fn format_duration(seconds: i32) -> String {
  let minutes = seconds / 60;
  let secs = seconds % 60;
  format!("{:02}:{:02}", minutes, secs)
}

async fn fetch_netease_cover_url(state: &AppState, title: &str, artist: &str) -> Option<String> {
  let keywords = format!("{} {}", title, artist).trim().to_string();
  if keywords.is_empty() || keywords == "未知" {
    return None;
  }
  let url = format!("{}{}", state.config.netease_base_url.trim_end_matches('/'), "/cloudsearch");
  let mut full_url = reqwest::Url::parse(&url).ok()?;
  {
    let mut pairs = full_url.query_pairs_mut();
    pairs.append_pair("keywords", &keywords);
    pairs.append_pair("limit", "1");
    pairs.append_pair("type", "1");
  }
  let resp = state.http.get(full_url)
    .header(header::ACCEPT_ENCODING, "identity")
    .send()
    .await
    .ok()?;
  let body = resp.bytes().await.ok()?;
  let json: serde_json::Value = serde_json::from_slice(&body).ok()?;
  let songs = json.get("result")?.get("songs")?.as_array()?;
  let first = songs.first()?;
  let album = first.get("al").or_else(|| first.get("album"));
  if let Some(url) = album.and_then(|v| v.get("picUrl")).and_then(|v| v.as_str()) {
    return Some(url.to_string());
  }
  if let Some(url) = first.get("picUrl").and_then(|v| v.as_str()) {
    return Some(url.to_string());
  }
  None
}


