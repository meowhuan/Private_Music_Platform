import { computed, onMounted, onBeforeUnmount, ref, watch } from "vue";

let sharedState;

const createState = () => {
  const apiBase = (import.meta.env.VITE_API_BASE || "").replace(/\/$/, "");
  const isNight = ref(false);
  const time = ref("");
  const date = ref("");

  const authToken = ref(typeof window !== "undefined" ? window.localStorage.getItem("music-token") || "" : "");
  const authLoading = ref(false);
  const authError = ref("");
  const authUsername = ref(typeof window !== "undefined" ? window.localStorage.getItem("music-username") || "" : "");
  const authIsAdmin = ref(typeof window !== "undefined" ? window.localStorage.getItem("music-is-admin") === "1" : false);
  const userCreateForm = ref({ username: "", password: "" });
  const userCreateLoading = ref(false);
  const userCreateError = ref("");
  const userCreateSuccess = ref("");
  const passwordForm = ref({ current: "", next: "" });
  const passwordLoading = ref(false);
  const passwordError = ref("");
  const passwordSuccess = ref("");
  const adminResetForm = ref({ username: "", next: "" });
  const adminResetLoading = ref(false);
  const adminResetError = ref("");
  const adminResetSuccess = ref("");
  const dataLoading = ref(false);
  const dataError = ref("");
  const lastFetchedAt = ref(0);
  const fetchCooldownMs = 20000;

  const loginForm = ref({
    username: "admin",
    password: ""
  });

  const nowPlaying = ref({
    title: "未播放",
    artist: "",
    album: "",
    length: "00:00",
    progress: 0,
    quality: "",
    mood: ""
  });

  const queue = ref([]);
  const playlists = ref([]);
  const library = ref([]);
  const devices = ref([]);
  const deviceCreateForm = ref({ name: "", desc: "" });
  const deviceCreateLoading = ref(false);
  const deviceCreateError = ref("");
  const deviceCreateResult = ref(null);
  const stats = ref([]);

  const uploadFile = ref(null);
  const uploadLyricFile = ref(null);
  const uploadCoverFile = ref(null);
  const uploadForm = ref({
    title: "",
    artist: "",
    album: "",
    tag: "离线"
  });
  const uploadLoading = ref(false);
  const uploadError = ref("");
  const uploadSuccess = ref("");

  const searchQuery = ref("");
  const searchLoading = ref(false);
  const searchError = ref("");
  const searchResults = ref([]);
  const selectedPlaylistId = ref("");
  const playlistDetail = ref(null);
  const playlistDetailLoading = ref(false);
  const playlistDetailError = ref("");
  const playlistDetailDurationPending = computed(() => {
    const tracks = playlistDetail.value?.tracks || [];
    return tracks.some((track) => {
      if (track.source !== "netease") return false;
      if (track.duration_seconds) return false;
      if (track.time && track.time !== "00:00") return false;
      return true;
    });
  });

  const playlistCreateForm = ref({
    name: "",
    desc: "",
    mood: ""
  });
  const playlistCreateLoading = ref(false);
  const playlistCreateError = ref("");

  const neteaseImport = ref({
    url: "",
    targetPlaylistId: "",
    loading: false,
    error: "",
    success: "",
    progress: ""
  });

  const audioEl = ref(null);
  const isPlaying = ref(false);
  const isBuffering = ref(false);
  const playError = ref("");
  const currentTrack = ref(null);
  const currentLyric = ref("");
  const currentLyricLoading = ref(false);
  const currentLyricError = ref("");
  const lyricLines = ref([]);
  const currentLyricIndex = ref(-1);
  const lastPersistAt = ref(0);
  const savedPlayback = ref(null);
  const lyricCache = ref({});
  const lastLyricPersistAt = ref(0);
  const queueDockOpen = ref(false);
  const uploadDockOpen = ref(false);
  const neteaseQualityLevel = ref("exhigh");
  const neteaseQualityMeta = ref(null);
  const durationSaved = ref({});
  const currentIndex = ref(-1);
  const durationSeconds = ref(0);
  const currentSeconds = ref(0);
  const volume = ref(0.8);
  const playMode = ref("order"); // order | repeat-one | repeat-all | shuffle
  const playbackTrend = ref({ label: "", labels: [], values: [] });
  const spectrum = ref(Array.from({ length: 24 }, () => 0));
  const prefetchIndex = ref(-1);
  const prefetching = ref(false);
  let spectrumCtx;
  let spectrumAnalyser;
  let spectrumSource;
  let spectrumRaf = 0;
  let spectrumDecayRaf = 0;
  let volumeFadeRaf = 0;

  const neteaseCookieMode = ref(typeof window !== "undefined" ? window.localStorage.getItem("netease-cookie-mode") || "admin" : "admin");
  const neteaseCookie = ref("");
  const neteaseProfile = ref(null);
  const neteaseLoginLoading = ref(false);
  const neteaseLoginError = ref("");
  const neteasePrompt = ref({
    show: false,
    message: "",
    action: ""
  });

  const qr = ref({
    key: "",
    qrimg: "",
    status: "idle",
    message: ""
  });
  const qrPolling = ref(false);
  const qrErrorCount = ref(0);
  const qrLast = ref({
    code: null,
    at: null,
    noCookie: false,
    raw: null
  });
  let qrTimer;

  const apiReady = computed(() => Boolean(apiBase));
  const apiBaseValid = computed(() => apiBase.startsWith("http://") || apiBase.startsWith("https://"));
  const isAuthed = computed(() => Boolean(authToken.value));
  const isAdmin = computed(() => {
    if (authIsAdmin.value) return true;
    const name = (authUsername.value || loginForm.value.username || "").trim();
    return name === "admin";
  });
  const resolveCoverUrl = (url) => {
    if (!url) return "";
    if (url.startsWith("http://") || url.startsWith("https://")) return url;
    if (url.startsWith("/") && apiBase) return `${apiBase}${url}`;
    return url;
  };
  const timeProgress = computed(() => {
    if (!durationSeconds.value) return "0%";
    return `${Math.min(100, Math.round((currentSeconds.value / durationSeconds.value) * 100))}%`;
  });
  const currentTimeText = computed(() => formatTime(currentSeconds.value));
  const durationText = computed(() => formatTime(durationSeconds.value));
  const prefetchCount = computed(() => (prefetchIndex.value >= 0 ? 1 : 0));
  const nextUpText = computed(() => {
    if (!queue.value.length || currentIndex.value < 0) return "";
    if (currentIndex.value + 1 >= queue.value.length && playMode.value !== "repeat-all") return "";
    if (!durationSeconds.value || !isPlaying.value) return "";
    const remaining = Math.max(0, Math.floor(durationSeconds.value - currentSeconds.value));
    return `下一首将于 ${formatTime(remaining)} 后开始，已缓存 ${prefetchCount.value} 首。`;
  });

  const syncDurationFromTrack = (track) => {
    const seconds = track?.duration_seconds;
    if (typeof seconds === "number" && seconds > 0) {
      durationSeconds.value = seconds;
    }
  };

  const fetchDurationIfMissing = async (item, force = false) => {
    if (!item) return;
    if (!force && (durationSeconds.value > 0 || item.duration_seconds)) return;
    if (item.source !== "netease") return;
    try {
      const detail = await apiGet("/api/netease/song/detail", { ids: item.source_id || item.id });
      const song = detail?.songs?.[0];
      const ms = song?.dt;
      if (typeof ms === "number" && ms > 0) {
        const seconds = Math.round(ms / 1000);
        durationSeconds.value = seconds;
        item.duration_seconds = seconds;
        item.time = formatTime(seconds);
        if (item.id && !item.id.startsWith("netease-")) {
          await fetch(apiUrl(`/api/tracks/${item.id}`), {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
              Authorization: `Bearer ${authToken.value}`
            },
            body: JSON.stringify({ duration_seconds: seconds })
          });
        }
      }
    } catch {}
  };

  const hydrateLibraryDurations = () => {
    if (!library.value?.length) return;
    let delay = 0;
    library.value.forEach((track) => {
      if (track.source !== "netease") return;
      if (track.duration_seconds) return;
      delay += 180;
      setTimeout(() => {
        fetchDurationIfMissing(track, true);
      }, delay);
    });
  };

  const hydratePlaylistDetailDurations = () => {
    const tracks = playlistDetail.value?.tracks || [];
    if (!tracks.length) return;
    let delay = 0;
    tracks.forEach((track) => {
      if (track.source !== "netease") return;
      if (track.duration_seconds) return;
      delay += 180;
      setTimeout(() => {
        fetchDurationIfMissing(track, true);
      }, delay);
    });
  };

  const ensureApiBase = () => {
    if (!apiBase) {
      throw new Error("请先设置 VITE_API_BASE");
    }
    if (!apiBaseValid.value) {
      throw new Error("VITE_API_BASE 必须包含 http:// 或 https://");
    }
  };

  const formatTime = (seconds) => {
    const safe = Math.max(0, Math.floor(seconds || 0));
    const m = Math.floor(safe / 60);
    const s = safe % 60;
    return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  };

  const requireNetease = (action) => {
    if (neteaseCookie.value) return true;
    const modeLabel = neteaseCookieMode.value === "user" ? "个人网易云账户（隔离）" : "管理员网易云账户（全局）";
    neteasePrompt.value = {
      show: true,
      message: `该操作需要${modeLabel}登录`,
      action:
        action ||
        (neteaseCookieMode.value === "user" ? "请先在设置页完成个人网易云扫码登录" : "请先在设置页完成管理员网易云扫码登录")
    };
    return false;
  };

  const buildNeteaseCookie = () => {
    if (!neteaseCookie.value) return "";
    if (neteaseCookie.value.includes("os=pc")) return neteaseCookie.value;
    return `${neteaseCookie.value}; os=pc`;
  };

  const dismissNeteasePrompt = () => {
    neteasePrompt.value = { show: false, message: "", action: "" };
  };

  const getNeteaseCookieKey = (modeOverride) => {
    const userKey = authUsername.value || loginForm.value.username || "user";
    const mode = modeOverride || neteaseCookieMode.value;
    return mode === "user" ? `netease-cookie:${userKey}` : "netease-cookie-admin";
  };

  const loadNeteaseCookie = () => {
    if (typeof window === "undefined") return;
    const store = window.localStorage;
    let cookie = store.getItem(getNeteaseCookieKey()) || "";
    if (!cookie && neteaseCookieMode.value === "admin") {
      cookie = store.getItem("netease-cookie-admin") || store.getItem("netease-cookie:admin") || "";
    }
    if (!cookie && neteaseCookieMode.value === "user") {
      const userKey = authUsername.value || loginForm.value.username || "user";
      cookie = store.getItem(`netease-cookie:${userKey}`) || "";
      if (!cookie && isAdmin.value) {
        const adminCookie = store.getItem("netease-cookie-admin") || store.getItem("netease-cookie:admin") || "";
        if (adminCookie) {
          neteaseCookieMode.value = "admin";
          store.setItem("netease-cookie-mode", "admin");
          cookie = adminCookie;
        }
      }
    }
    neteaseCookie.value = cookie;
  };

  const setNeteaseCookie = (cookie) => {
    neteaseCookie.value = cookie || "";
    if (typeof window !== "undefined") {
      window.localStorage.setItem(getNeteaseCookieKey(), neteaseCookie.value);
    }
  };

  const setNeteaseCookieMode = (mode) => {
    neteaseCookieMode.value = mode === "user" ? "user" : "admin";
    if (typeof window !== "undefined") {
      window.localStorage.setItem("netease-cookie-mode", neteaseCookieMode.value);
    }
    loadNeteaseCookie();
    neteaseProfile.value = null;
  };

  const apiUrl = (path, params) => {
    ensureApiBase();
    const url = new URL(path, apiBase);
    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        if (value !== undefined && value !== null && value !== "") {
          url.searchParams.set(key, value);
        }
      });
    }
    return url.toString();
  };

  const apiGet = async (path, params) => {
    ensureApiBase();
    const response = await fetch(apiUrl(path, params), {
      headers: {
        Authorization: `Bearer ${authToken.value}`
      }
    });
    if (!response.ok) {
      const text = await response.text().catch(() => "");
      let message = `请求失败 (${response.status})`;
      if (text) {
        try {
          const parsed = JSON.parse(text);
          message = parsed.message || message;
        } catch {
          message = text;
        }
      }
      throw new Error(message);
    }
    return response.json();
  };

  const apiGetText = async (path, params) => {
    ensureApiBase();
    const response = await fetch(apiUrl(path, params), {
      headers: {
        Authorization: `Bearer ${authToken.value}`
      }
    });
    if (!response.ok) {
      const text = await response.text().catch(() => "");
      let message = `请求失败 (${response.status})`;
      if (text) {
        message = text;
      }
      throw new Error(message);
    }
    return response.text();
  };

  const updateTime = () => {
    const now = new Date();
    time.value = now.toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit", second: "2-digit" });
    date.value = now.toLocaleDateString("zh-CN", { month: "long", day: "numeric", weekday: "short" });
  };

  const formatRelativeTime = (value) => {
    if (!value) return "未连接";
    const safe = String(value).trim();
    const parsed = new Date(safe.includes("T") ? safe : safe.replace(" ", "T"));
    if (Number.isNaN(parsed.getTime())) return safe;
    const diffMs = Date.now() - parsed.getTime();
    const diffMin = Math.floor(diffMs / 60000);
    if (diffMin < 1) return "刚刚";
    if (diffMin < 60) return `${diffMin} 分钟前`;
    const diffHour = Math.floor(diffMin / 60);
    if (diffHour < 24) return `${diffHour} 小时前`;
    const diffDay = Math.floor(diffHour / 24);
    return `${diffDay} 天前`;
  };

  const applyTheme = (next) => {
    isNight.value = next;
    if (typeof window !== "undefined") {
      window.localStorage.setItem("music-theme", next ? "night" : "day");
    }
  };

  const toggleTheme = () => {
    applyTheme(!isNight.value);
  };

  const fetchAll = async (force = false) => {
    if (!authToken.value) return;
    if (!force && Date.now() - lastFetchedAt.value < fetchCooldownMs) return;
    dataError.value = "";
    dataLoading.value = true;
    try {
      const [
        nowPlayingResp,
        queueResp,
        playlistsResp,
        libraryResp,
        devicesResp,
        statsResp,
        trendResp
      ] = await Promise.all([
        apiGet("/api/now-playing"),
        apiGet("/api/queue"),
        apiGet("/api/playlists"),
        apiGet("/api/library", { limit: 12 }),
        apiGet("/api/devices"),
        apiGet("/api/stats"),
        apiGet("/api/playback/trend", { days: 9 })
      ]);
      nowPlaying.value = nowPlayingResp;
      if (nowPlayingResp) {
        syncDurationFromTrack(nowPlayingResp);
      }
      queue.value = queueResp;
      if (!queueResp?.length) {
        currentIndex.value = -1;
      } else if (currentIndex.value < 0) {
        currentIndex.value = 0;
      }
      if (queueResp?.length) {
        const neteaseItems = queueResp.filter((item) => item.source === "netease" && (item.source_id || item.id));
        if (neteaseItems.length) {
          preloadLyricBatch(neteaseItems.slice(0, 3));
        }
      }
      playlists.value = playlistsResp;
      if (!selectedPlaylistId.value && playlistsResp?.length) {
        selectedPlaylistId.value = playlistsResp[0].id;
      }
      library.value = libraryResp;
      hydrateLibraryDurations();
      devices.value = devicesResp;
      stats.value = statsResp;
      playbackTrend.value = trendResp || playbackTrend.value;
      lastFetchedAt.value = Date.now();
    } catch (err) {
      dataError.value = err.message || "数据获取失败";
    } finally {
      dataLoading.value = false;
    }
  };

  watch(
    () => [neteaseCookieMode.value, authUsername.value],
    () => {
      loadNeteaseCookie();
    },
    { immediate: true }
  );

  const createDevice = async () => {
    deviceCreateError.value = "";
    deviceCreateResult.value = null;
    if (!deviceCreateForm.value.name.trim()) {
      deviceCreateError.value = "请输入设备名称";
      return;
    }
    deviceCreateLoading.value = true;
    try {
      const resp = await fetch(apiUrl("/api/devices"), {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${authToken.value}`
        },
        body: JSON.stringify({
          name: deviceCreateForm.value.name.trim(),
          desc: deviceCreateForm.value.desc || ""
        })
      });
      if (!resp.ok) {
        const err = await resp.json().catch(() => ({}));
        throw new Error(err.message || "创建设备失败");
      }
      deviceCreateResult.value = await resp.json();
      deviceCreateForm.value = { name: "", desc: "" };
      await fetchAll(true);
    } catch (err) {
      deviceCreateError.value = err.message || "创建设备失败";
    } finally {
      deviceCreateLoading.value = false;
    }
  };

  const syncDevice = async (id) => {
    if (!id) return;
    await fetch(apiUrl(`/api/devices/${id}/sync`), {
      method: "POST",
      headers: { Authorization: `Bearer ${authToken.value}` }
    });
    await fetchAll(true);
  };

  const deleteDevice = async (id) => {
    if (!id) return;
    await fetch(apiUrl(`/api/devices/${id}`), {
      method: "DELETE",
      headers: { Authorization: `Bearer ${authToken.value}` }
    });
    await fetchAll(true);
  };

  const login = async () => {
    authLoading.value = true;
    authError.value = "";
    try {
      ensureApiBase();
      const response = await fetch(apiUrl("/api/auth/login"), {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(loginForm.value)
      });
      if (!response.ok) {
        const error = await response.json().catch(() => ({}));
        throw new Error(error.message || "登录失败");
      }
      const data = await response.json();
      authToken.value = data.token;
      authUsername.value = data.user?.username || loginForm.value.username;
      authIsAdmin.value = Boolean(data.user?.is_admin);
      if (typeof window !== "undefined") {
        window.localStorage.setItem("music-token", data.token);
        window.localStorage.setItem("music-username", authUsername.value);
        window.localStorage.setItem("music-is-admin", authIsAdmin.value ? "1" : "0");
      }
      loadNeteaseCookie();
      await fetchAll(true);
    } catch (err) {
      authError.value = err.message || "登录失败";
    } finally {
      authLoading.value = false;
    }
  };

  const changePassword = async () => {
    passwordError.value = "";
    passwordSuccess.value = "";
    if (!passwordForm.value.current || !passwordForm.value.next) {
      passwordError.value = "请填写当前密码与新密码";
      return;
    }
    if (passwordForm.value.next.length < 6) {
      passwordError.value = "新密码至少 6 位";
      return;
    }
    passwordLoading.value = true;
    try {
      const response = await fetch(apiUrl("/api/users/password"), {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${authToken.value}`
        },
        body: JSON.stringify({
          current_password: passwordForm.value.current,
          new_password: passwordForm.value.next
        })
      });
      if (!response.ok) {
        const error = await response.json().catch(() => ({}));
        throw new Error(error.message || "修改失败");
      }
      passwordSuccess.value = "密码已更新";
      passwordForm.value.current = "";
      passwordForm.value.next = "";
    } catch (err) {
      passwordError.value = err.message || "修改失败";
    } finally {
      passwordLoading.value = false;
    }
  };

  const adminResetPassword = async () => {
    adminResetError.value = "";
    adminResetSuccess.value = "";
    if (!adminResetForm.value.username || !adminResetForm.value.next) {
      adminResetError.value = "请填写用户名与新密码";
      return;
    }
    if (adminResetForm.value.next.length < 6) {
      adminResetError.value = "新密码至少 6 位";
      return;
    }
    adminResetLoading.value = true;
    try {
      const response = await fetch(apiUrl("/api/users/password/reset"), {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${authToken.value}`
        },
        body: JSON.stringify({
          username: adminResetForm.value.username,
          new_password: adminResetForm.value.next
        })
      });
      if (!response.ok) {
        const error = await response.json().catch(() => ({}));
        throw new Error(error.message || "重置失败");
      }
      adminResetSuccess.value = "密码已重置";
      adminResetForm.value.username = "";
      adminResetForm.value.next = "";
    } catch (err) {
      adminResetError.value = err.message || "重置失败";
    } finally {
      adminResetLoading.value = false;
    }
  };

  const createUser = async () => {
    userCreateError.value = "";
    userCreateSuccess.value = "";
    if (!userCreateForm.value.username.trim() || !userCreateForm.value.password.trim()) {
      userCreateError.value = "请输入用户名与密码";
      return;
    }
    userCreateLoading.value = true;
    try {
      const resp = await fetch(apiUrl("/api/users"), {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${authToken.value}`
        },
        body: JSON.stringify({
          username: userCreateForm.value.username.trim(),
          password: userCreateForm.value.password.trim()
        })
      });
      if (!resp.ok) {
        const err = await resp.json().catch(() => ({}));
        throw new Error(err.message || "注册失败");
      }
      await resp.json();
      userCreateSuccess.value = "用户已创建";
      userCreateForm.value = { username: "", password: "" };
    } catch (err) {
      userCreateError.value = err.message || "注册失败";
    } finally {
      userCreateLoading.value = false;
    }
  };

  const logout = () => {
    authToken.value = "";
    authUsername.value = "";
    authIsAdmin.value = false;
    if (typeof window !== "undefined") {
      window.localStorage.removeItem("music-token");
      window.localStorage.removeItem("music-username");
      window.localStorage.removeItem("music-is-admin");
    }
    lyricCache.value = {};
    if (typeof window !== "undefined") {
      window.localStorage.removeItem("lyric-cache");
      window.localStorage.removeItem("music-playback");
      window.localStorage.removeItem("music-playstate");
      window.localStorage.removeItem("music-queue");
    }
  };

  const onFileChange = (event) => {
    const files = Array.from(event?.dataTransfer?.files || event?.target?.files || []);
    if (!files.length) {
      uploadFile.value = null;
      return;
    }
    const isAudio = (name, type) => {
      if (type && type.startsWith("audio/")) return true;
      return /\.(mp3|flac|m4a|wav|ogg|aac)$/i.test(name);
    };
    const audio = files.find((file) => isAudio(file.name, file.type)) || files[0];
    uploadFile.value = audio || null;
    const base = audio?.name ? audio.name.replace(/\.[^.]+$/, "") : "";
    const lyric = files.find((file) => /\.(lrc|txt)$/i.test(file.name) && (!base || file.name.replace(/\.[^.]+$/, "") === base));
    if (lyric) {
      uploadLyricFile.value = lyric;
    }
  };

  const onLyricFileChange = (event) => {
    const file = event.target.files?.[0];
    uploadLyricFile.value = file || null;
  };

  const onCoverFileChange = (event) => {
    const file = event.target.files?.[0];
    uploadCoverFile.value = file || null;
  };

  const uploadMusic = async () => {
    uploadError.value = "";
    uploadSuccess.value = "";
    if (!uploadFile.value) {
      uploadError.value = "请选择音乐文件";
      return;
    }
    uploadLoading.value = true;
    try {
      ensureApiBase();
    const form = new FormData();
    form.append("file", uploadFile.value);
    if (uploadLyricFile.value) {
      form.append("lyric", uploadLyricFile.value);
    }
    if (uploadCoverFile.value) {
      form.append("cover", uploadCoverFile.value);
    }
      if (uploadForm.value.title) form.append("title", uploadForm.value.title);
      if (uploadForm.value.artist) form.append("artist", uploadForm.value.artist);
      if (uploadForm.value.album) form.append("album", uploadForm.value.album);
      if (uploadForm.value.tag) form.append("tag", uploadForm.value.tag);

      const response = await fetch(apiUrl("/api/upload/music"), {
        method: "POST",
        headers: {
          Authorization: `Bearer ${authToken.value}`
        },
        body: form
      });
      if (!response.ok) {
        const error = await response.json().catch(() => ({}));
        throw new Error(error.message || "上传失败");
      }
      await response.json();
      uploadSuccess.value = "上传成功";
      uploadFile.value = null;
      uploadLyricFile.value = null;
      uploadCoverFile.value = null;
      uploadForm.value.title = "";
      uploadForm.value.artist = "";
      uploadForm.value.album = "";
      await fetchAll();
    } catch (err) {
      uploadError.value = err.message || "上传失败";
    } finally {
      uploadLoading.value = false;
    }
  };

  const queueReplaceWithPlaylist = async (playlistId) => {
    if (!playlistId) return;
    dataLoading.value = true;
    try {
      await fetch(apiUrl("/api/queue/replace"), {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${authToken.value}`
        },
        body: JSON.stringify({ playlist_id: playlistId })
      });
      await fetchAll(true);
      await playQueueIndex(0);
    } finally {
      dataLoading.value = false;
    }
  };

  const queueClearAll = async () => {
    await fetch(apiUrl("/api/queue/clear"), {
      method: "POST",
      headers: {
        Authorization: `Bearer ${authToken.value}`
      }
    });
    queue.value = [];
    currentIndex.value = -1;
  };

  const queueAddItems = async (items) => {
    if (!items?.length) return;
    if (items.some((item) => item.source === "netease") && !requireNetease("加入网易云曲目到队列")) {
      return;
    }
    await fetch(apiUrl("/api/queue/add"), {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${authToken.value}`
      },
      body: JSON.stringify({ items })
    });
    await fetchAll(true);
  };

  const postAddTrackToPlaylist = async (playlistId, item) => {
    await fetch(apiUrl(`/api/playlists/${playlistId}/tracks`), {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${authToken.value}`
      },
      body: JSON.stringify({
        track_id: item.id?.startsWith("netease-") ? undefined : item.id,
        source: item.source,
        source_id: item.source_id,
        title: item.title,
        artist: item.artist,
        album: item.album,
        cover_url: item.cover_url
      })
    });
  };

  const addTrackToPlaylist = async (playlistId, item) => {
    if (!playlistId) return;
    if (item.source === "netease" && !requireNetease("添加网易云曲目到歌单")) return;
    await postAddTrackToPlaylist(playlistId, item);
    await fetchAll(true);
  };

  const createPlaylist = async () => {
    playlistCreateError.value = "";
    if (!playlistCreateForm.value.name.trim()) {
      playlistCreateError.value = "请输入歌单名称";
      return;
    }
    playlistCreateLoading.value = true;
    try {
      await fetch(apiUrl("/api/playlists"), {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${authToken.value}`
        },
        body: JSON.stringify({
          name: playlistCreateForm.value.name.trim(),
          desc: playlistCreateForm.value.desc || "",
          mood: playlistCreateForm.value.mood || ""
        })
      });
      playlistCreateForm.value = { name: "", desc: "", mood: "" };
      await fetchAll(true);
    } catch (err) {
      playlistCreateError.value = err.message || "新建失败";
    } finally {
      playlistCreateLoading.value = false;
    }
  };

  const deletePlaylist = async (playlistId) => {
    if (!playlistId) return;
    await fetch(apiUrl(`/api/playlists/${playlistId}`), {
      method: "DELETE",
      headers: {
        Authorization: `Bearer ${authToken.value}`
      }
    });
    if (selectedPlaylistId.value === playlistId) {
      selectedPlaylistId.value = "";
    }
    await fetchAll(true);
  };

  const removeTrackFromPlaylist = async (playlistId, trackId) => {
    if (!playlistId || !trackId) return;
    await fetch(apiUrl(`/api/playlists/${playlistId}/tracks/${trackId}`), {
      method: "POST",
      headers: {
        Authorization: `Bearer ${authToken.value}`
      }
    });
    await fetchAll(true);
    if (playlistDetail.value?.id === playlistId) {
      await fetchPlaylistDetail(playlistId);
    }
  };

  const deleteTrack = async (trackId) => {
    if (!trackId) return;
    await fetch(apiUrl(`/api/tracks/${trackId}`), {
      method: "DELETE",
      headers: {
        Authorization: `Bearer ${authToken.value}`
      }
    });
    await fetchAll(true);
    if (playlistDetail.value?.tracks) {
      playlistDetail.value = {
        ...playlistDetail.value,
        tracks: playlistDetail.value.tracks.filter((track) => track.id !== trackId)
      };
    }
  };

  const fetchPlaylistDetail = async (playlistId) => {
    if (!playlistId) return;
    playlistDetailLoading.value = true;
    playlistDetailError.value = "";
    try {
      const data = await apiGet(`/api/playlists/${playlistId}`);
      playlistDetail.value = data;
      hydratePlaylistDetailDurations();
    } catch (err) {
      playlistDetailError.value = err.message || "获取歌单失败";
    } finally {
      playlistDetailLoading.value = false;
    }
  };

  const queueReorder = async (order) => {
    if (!order?.length) return;
    await fetch(apiUrl("/api/queue/reorder"), {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${authToken.value}`
      },
      body: JSON.stringify({ order })
    });
    await fetchAll(true);
  };

  const searchNetease = async () => {
    searchError.value = "";
    searchResults.value = [];
    if (!requireNetease("网易云搜索")) return;
    if (!searchQuery.value.trim()) {
      searchError.value = "请输入关键词";
      return;
    }
    searchLoading.value = true;
    try {
      const data = await apiGet("/api/netease/search", {
        keywords: searchQuery.value.trim(),
        limit: 8,
        cookie: buildNeteaseCookie() || undefined
      });
      const songs = data?.result?.songs || [];
      searchResults.value = songs.map((song) => ({
        id: song.id,
        name: song.name,
        artists: song.ar?.map((item) => item.name).join(" / ") || "",
        album: song.al?.name || "",
        cover_url: song.al?.picUrl || ""
      }));
    } catch (err) {
      searchError.value = err.message || "搜索失败";
    } finally {
      searchLoading.value = false;
    }
  };

  const extractPlaylistId = (input) => {
    if (!input) return "";
    const raw = String(input).trim();
    const idMatch = raw.match(/(?:id=|playlist\/)(\d+)/i);
    if (idMatch?.[1]) return idMatch[1];
    const digits = raw.match(/\d{5,}/);
    return digits?.[0] || "";
  };

  const importNeteasePlaylist = async () => {
    neteaseImport.value.error = "";
    neteaseImport.value.success = "";
    neteaseImport.value.progress = "";
    if (!requireNetease("导入网易云歌单")) return;
    const playlistId = extractPlaylistId(neteaseImport.value.url);
    if (!playlistId) {
      neteaseImport.value.error = "请输入网易云歌单链接或 ID";
      return;
    }
    neteaseImport.value.loading = true;
    try {
      const detail = await apiGet("/api/netease/playlist/detail", {
        id: playlistId,
        timestamp: Date.now(),
        cookie: buildNeteaseCookie() || undefined
      });
      const playlistInfo = detail?.playlist || {};
      const trackCount = playlistInfo.trackCount || playlistInfo.tracks?.length || 0;

      let targetId = neteaseImport.value.targetPlaylistId;
      if (!targetId) {
        const createResp = await fetch(apiUrl("/api/playlists"), {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${authToken.value}`
          },
          body: JSON.stringify({
            name: playlistInfo.name || "网易云歌单",
            desc: playlistInfo.description || "",
            mood: "网易云"
          })
        });
        if (!createResp.ok) {
          const err = await createResp.json().catch(() => ({}));
          throw new Error(err.message || "创建歌单失败");
        }
        const created = await createResp.json();
        targetId = created.id;
      }

      const songs = [];
      const limit = 200;
      let offset = 0;
      while (offset < trackCount || offset === 0) {
        const chunk = await apiGet("/api/netease/playlist/track/all", {
          id: playlistId,
          limit,
          offset,
          timestamp: Date.now(),
          cookie: buildNeteaseCookie() || undefined
        });
        const list = chunk?.songs || [];
        if (!list.length) break;
        songs.push(...list);
        offset += list.length;
        neteaseImport.value.progress = `已拉取 ${songs.length}/${trackCount || songs.length} 首`;
        if (list.length < limit) break;
      }

      let imported = 0;
      for (const song of songs) {
        await postAddTrackToPlaylist(targetId, {
          source: "netease",
          source_id: String(song.id),
          title: song.name,
          artist: song.ar?.map((item) => item.name).join(" / ") || "",
          album: song.al?.name || "",
          cover_url: song.al?.picUrl || ""
        });
        imported += 1;
        if (imported % 20 === 0) {
          neteaseImport.value.progress = `正在导入 ${imported}/${songs.length} 首`;
        }
      }

      await fetchAll(true);
      neteaseImport.value.success = `导入完成，共 ${imported} 首`;
      neteaseImport.value.progress = "";
    } catch (err) {
      neteaseImport.value.error = err.message || "导入失败";
    } finally {
      neteaseImport.value.loading = false;
    }
  };

  const playFromNetease = async (track) => {
    playError.value = "";
    if (!track) return;
    if (!requireNetease("网易云播放")) return;
    await playTrackItem({
      id: `netease-${track.id}`,
      source: "netease",
      source_id: String(track.id),
      title: track.name,
      artist: track.artists,
      album: track.album,
      cover_url: track.cover_url
    });
  };

  const resolveTrackUrl = async (item) => {
    if (item.source === "local") {
      return apiUrl(`/api/stream/${item.id}`);
    }
    if (!requireNetease("网易云播放")) return null;
    try {
      const data = await apiGet("/api/netease/song/url/v1", {
        id: item.source_id || item.id,
        level: neteaseQualityLevel.value || "exhigh",
        unblock: "true",
        timestamp: Date.now(),
        cookie: buildNeteaseCookie() || undefined
      });
      const url = data?.data?.[0]?.url;
      if (url) return url;
    } catch {}
    return apiUrl("/api/netease/stream", {
      id: item.source_id || item.id,
      level: neteaseQualityLevel.value || "exhigh",
      unblock: "true",
      cookie: buildNeteaseCookie() || undefined
    });
  };

  const prefetchNextTrack = async () => {
    if (prefetching.value) return;
    if (!queue.value.length) return;
    if (playMode.value === "shuffle" || playMode.value === "repeat-one") return;
    const nextIndex = currentIndex.value + 1 < queue.value.length ? currentIndex.value + 1 : (playMode.value === "repeat-all" ? 0 : -1);
    if (nextIndex < 0) return;
    if (prefetchIndex.value === nextIndex) return;
    const nextItem = queue.value[nextIndex];
    if (!nextItem) return;
    if (nextItem.source === "netease" && !neteaseCookie.value) return;
    prefetching.value = true;
    prefetchIndex.value = nextIndex;
    try {
      const url = await resolveTrackUrl(nextItem);
      if (!url) return;
      await fetch(url, { headers: { Range: "bytes=0-1" } });
    } catch {
    } finally {
      prefetching.value = false;
    }
  };

  const fetchLyric = async (item) => {
    currentLyric.value = "";
    currentLyricError.value = "";
    lyricLines.value = [];
    currentLyricIndex.value = -1;
    if (!item) return;
    if (item.source === "local") {
      const cacheKey = item.id;
      if (cacheKey && lyricCache.value[cacheKey]) {
        const cached = lyricCache.value[cacheKey];
        currentLyric.value = cached.raw || "";
        lyricLines.value = cached.lines || [];
        return;
      }
      currentLyricLoading.value = true;
      try {
        const text = await apiGetText(`/api/lyric/local/${item.id}`);
        currentLyric.value = text || "";
        lyricLines.value = parseLyricLines(currentLyric.value, "");
        if (cacheKey) {
          setLyricCache(cacheKey, {
            raw: currentLyric.value,
            lines: lyricLines.value
          });
        }
      } catch (err) {
        currentLyricError.value = err.message || "暂无本地歌词";
      } finally {
        currentLyricLoading.value = false;
      }
      return;
    }
    if (item.source !== "netease") return;
    if (!requireNetease("获取网易云歌词")) return;
    const cacheKey = item.source_id || item.id;
    if (cacheKey && lyricCache.value[cacheKey]) {
      const cached = lyricCache.value[cacheKey];
      currentLyric.value = cached.raw || "";
      lyricLines.value = cached.lines || [];
      return;
    }
    currentLyricLoading.value = true;
    try {
      const data = await apiGet("/api/netease/lyric/new", {
        id: item.source_id || item.id,
        timestamp: Date.now(),
        cookie: buildNeteaseCookie() || undefined
      });
      let lyricText = data?.lrc?.lyric || data?.yrc?.lyric || "";
      let transText = data?.tlyric?.lyric || "";
      if (!lyricText) {
        const fallback = await apiGet("/api/netease/lyric", {
          id: item.source_id || item.id,
          timestamp: Date.now(),
          cookie: buildNeteaseCookie() || undefined
        });
        lyricText = fallback?.lrc?.lyric || "";
        transText = fallback?.tlyric?.lyric || "";
      } else if (!transText) {
        const fallback = await apiGet("/api/netease/lyric", {
          id: item.source_id || item.id,
          timestamp: Date.now(),
          cookie: buildNeteaseCookie() || undefined
        });
        transText = fallback?.tlyric?.lyric || "";
      }
      currentLyric.value = lyricText || "";
      lyricLines.value = parseLyricLines(currentLyric.value, transText);
      if (cacheKey) {
        setLyricCache(cacheKey, {
          raw: currentLyric.value,
          lines: lyricLines.value
        });
      }
    } catch (err) {
      currentLyricError.value = err.message || "歌词获取失败";
    } finally {
      currentLyricLoading.value = false;
    }
  };

  const preloadLyricForItem = async (item) => {
    if (!item) return;
    if (item.source === "local") {
      const cacheKey = item.id;
      if (!cacheKey || lyricCache.value[cacheKey]) return;
      try {
        const text = await apiGetText(`/api/lyric/local/${item.id}`);
        const lines = parseLyricLines(text || "", "");
        setLyricCache(cacheKey, { raw: text || "", lines });
      } catch {}
      return;
    }
    if (item.source !== "netease") return;
    if (!neteaseCookie.value) return;
    const cacheKey = item.source_id || item.id;
    if (!cacheKey || lyricCache.value[cacheKey]) return;
    try {
      const data = await apiGet("/api/netease/lyric/new", {
        id: cacheKey,
        timestamp: Date.now(),
        cookie: buildNeteaseCookie() || undefined
      });
      let lyricText = data?.lrc?.lyric || data?.yrc?.lyric || "";
      let transText = data?.tlyric?.lyric || "";
      if (!lyricText || !transText) {
        const fallback = await apiGet("/api/netease/lyric", {
          id: cacheKey,
          timestamp: Date.now(),
          cookie: buildNeteaseCookie() || undefined
        });
        if (!lyricText) lyricText = fallback?.lrc?.lyric || "";
        if (!transText) transText = fallback?.tlyric?.lyric || "";
      }
      const lines = parseLyricLines(lyricText || "", transText || "");
      setLyricCache(cacheKey, { raw: lyricText || "", lines });
    } catch {}
  };

  const preloadLyricBatch = (items) => {
    if (!items?.length) return;
    let delay = 0;
    items.forEach((item) => {
      delay += 120;
      setTimeout(() => {
        preloadLyricForItem(item);
      }, delay);
    });
  };

  const setLyricCache = (key, payload) => {
    if (!key) return;
    const next = { ...lyricCache.value };
    next[key] = { ...payload, at: Date.now() };
    const keys = Object.keys(next);
    if (keys.length > 50) {
      keys
        .sort((a, b) => (next[a]?.at || 0) - (next[b]?.at || 0))
        .slice(0, keys.length - 50)
        .forEach((k) => delete next[k]);
    }
    lyricCache.value = next;
    persistLyricCache();
  };

  const persistLyricCache = () => {
    if (typeof window === "undefined") return;
    const now = Date.now();
    if (now - lastLyricPersistAt.value < 2000) return;
    lastLyricPersistAt.value = now;
    window.localStorage.setItem("lyric-cache", JSON.stringify(lyricCache.value));
  };

  const restoreLyricCache = () => {
    if (typeof window === "undefined") return;
    const saved = window.localStorage.getItem("lyric-cache");
    if (!saved) return;
    try {
      lyricCache.value = JSON.parse(saved) || {};
    } catch {}
  };

  watch(
    [queue, neteaseCookie],
    ([nextQueue]) => {
      if (!nextQueue?.length) return;
      const neteaseItems = nextQueue.filter((item) => item.source === "netease" && (item.source_id || item.id));
      if (neteaseItems.length) {
        preloadLyricBatch(neteaseItems.slice(0, 3));
      }
    },
    { deep: true }
  );

  const parseLyricLines = (raw, transRaw) => {
    if (!raw) return [];
    const lines = raw.split("\n");
    const transMap = buildLyricMap(transRaw);
    const parsed = [];
    const timeRe = /\[(\d{1,2}):(\d{2})(?:\.(\d{1,3}))?\]/g;
    for (const line of lines) {
      if (!line || /^\s*\{.*\}\s*$/.test(line)) continue;
      let match;
      const text = line.replace(timeRe, "").trim();
      if (!text) continue;
      while ((match = timeRe.exec(line)) !== null) {
        const m = Number(match[1] || 0);
        const s = Number(match[2] || 0);
        const ms = Number((match[3] || "0").padEnd(3, "0"));
        const time = m * 60 + s + ms / 1000;
        const trans = transMap.get(time) || "";
        parsed.push({ time, text, trans });
      }
    }
    return parsed.sort((a, b) => a.time - b.time);
  };

  const buildLyricMap = (raw) => {
    const map = new Map();
    if (!raw) return map;
    const lines = raw.split("\n");
    const timeRe = /\[(\d{1,2}):(\d{2})(?:\.(\d{1,3}))?\]/g;
    for (const line of lines) {
      if (!line || /^\s*\{.*\}\s*$/.test(line)) continue;
      let match;
      const text = line.replace(timeRe, "").trim();
      if (!text) continue;
      while ((match = timeRe.exec(line)) !== null) {
        const m = Number(match[1] || 0);
        const s = Number(match[2] || 0);
        const ms = Number((match[3] || "0").padEnd(3, "0"));
        const time = m * 60 + s + ms / 1000;
        map.set(time, text);
      }
    }
    return map;
  };

  const playTrackItem = async (item, index = null, options = {}) => {
    playError.value = "";
    if (!item) return;
    durationSeconds.value = 0;
    syncDurationFromTrack(item);
    if (options.smoothTransition) {
      await fadeOutAndPause(260);
    }
    if (item.source === "netease") {
      neteaseQualityLevel.value = "exhigh";
      await loadNeteaseQuality(item);
    }
    const url = await resolveTrackUrl(item);
    if (!url) {
      playError.value = "获取播放地址失败";
      return;
    }
    if (!audioEl.value) return;
    isBuffering.value = true;
    audioEl.value.src = url;
    audioEl.value.load();
    audioEl.value.volume = volume.value;
    await fetchDurationIfMissing(item);
    const waitForBuffer = (seconds, timeoutMs = 4000) =>
      new Promise((resolve) => {
        const start = performance.now();
        const tick = () => {
          const buffered = audioEl.value?.buffered;
          const current = audioEl.value?.currentTime || 0;
          let ok = false;
          if (buffered && buffered.length) {
            const end = buffered.end(buffered.length - 1);
            ok = end - current >= seconds;
          }
          if (ok || performance.now() - start > timeoutMs) {
            resolve(null);
          } else {
            requestAnimationFrame(tick);
          }
        };
        requestAnimationFrame(tick);
      });
    if (options.prebuffer) {
      await waitForBuffer(5);
    }
    try {
      preloadLyricForItem(item);
      if (savedPlayback.value && item) {
        const matchId = savedPlayback.value?.track?.id === item.id;
        const matchSourceId = savedPlayback.value?.track?.source_id && item.source_id && savedPlayback.value.track.source_id === item.source_id;
        if (matchId || matchSourceId) {
          const seekTo = Number(savedPlayback.value.seconds || 0);
          if (seekTo > 0) {
            const once = () => {
              if (audioEl.value) {
                audioEl.value.currentTime = Math.min(seekTo, audioEl.value.duration || seekTo);
              }
              audioEl.value?.removeEventListener("loadedmetadata", once);
            };
            audioEl.value.addEventListener("loadedmetadata", once);
          }
        }
      }
      audioEl.value.volume = options.fade === false ? volume.value : 0;
      await audioEl.value.play();
      initSpectrum();
      if (spectrumCtx?.state === "suspended") {
        spectrumCtx.resume().catch(() => {});
      }
      isPlaying.value = true;
      startSpectrum();
      if (options.fade !== false) {
        fadeVolume(0, volume.value, 320);
      }
      recordPlayback(item);
      currentTrack.value = item;
      if (index !== null) currentIndex.value = index;
      await fetchLyric(item);
    } catch {
      if (item.source === "netease") {
        try {
          neteaseQualityLevel.value = "standard";
          await loadNeteaseQuality(item);
          const fallback = apiUrl("/api/netease/stream", {
            id: item.source_id || item.id,
            level: neteaseQualityLevel.value,
            unblock: "true",
            cookie: buildNeteaseCookie() || undefined
          });
          audioEl.value.src = fallback;
          audioEl.value.load();
          audioEl.value.volume = options.fade === false ? volume.value : 0;
          await audioEl.value.play();
          initSpectrum();
          if (spectrumCtx?.state === "suspended") {
            spectrumCtx.resume().catch(() => {});
          }
          isPlaying.value = true;
          startSpectrum();
          if (options.fade !== false) {
            fadeVolume(0, volume.value, 320);
          }
          recordPlayback(item);
          currentTrack.value = item;
          if (index !== null) currentIndex.value = index;
          await fetchLyric(item);
          return;
        } catch {}
      }
      isPlaying.value = false;
      playError.value = "播放失败";
      currentTrack.value = null;
    } finally {
      isBuffering.value = false;
    }
  };

  const prepareTrackForResume = async (item, seconds = 0) => {
    if (!item || !audioEl.value) return;
    syncDurationFromTrack(item);
    if (item.source === "netease") {
      neteaseQualityLevel.value = neteaseQualityLevel.value || "exhigh";
      await loadNeteaseQuality(item);
    }
    const url = await resolveTrackUrl(item);
    if (!url) return;
    audioEl.value.src = url;
    audioEl.value.load();
    if (seconds > 0) {
      const once = () => {
        if (audioEl.value) {
          audioEl.value.currentTime = Math.min(seconds, audioEl.value.duration || seconds);
        }
        audioEl.value?.removeEventListener("loadedmetadata", once);
      };
      audioEl.value.addEventListener("loadedmetadata", once);
    }
    audioEl.value.pause();
    isPlaying.value = false;
  };

  const playQueueIndex = async (index, options = {}) => {
    if (index < 0) return;
    if (index >= queue.value.length) {
      if (authToken.value) {
        await fetchAll(true);
      }
    }
    const item = queue.value[index];
    if (!item) {
      playError.value = "播放队列为空";
      return;
    }
    await playTrackItem(item, index, options);
  };

  const playNext = async (manual = false) => {
    if (!queue.value.length) {
      playError.value = "播放队列为空";
      return;
    }
    if (playMode.value === "repeat-one") {
      await playQueueIndex(currentIndex.value >= 0 ? currentIndex.value : 0, {
        smoothTransition: !manual,
        fade: !manual,
        prebuffer: manual
      });
      return;
    }
    if (playMode.value === "shuffle") {
      const next = Math.floor(Math.random() * queue.value.length);
      await playQueueIndex(next, { smoothTransition: !manual, fade: !manual, prebuffer: manual });
      return;
    }
    const nextIndex = currentIndex.value + 1;
    if (nextIndex < queue.value.length) {
      await playQueueIndex(nextIndex, { smoothTransition: !manual, fade: !manual, prebuffer: manual });
    } else if (playMode.value === "repeat-all") {
      await playQueueIndex(0, { smoothTransition: !manual, fade: !manual, prebuffer: manual });
    }
  };

  const playPrev = async (manual = false) => {
    if (!queue.value.length) {
      playError.value = "播放队列为空";
      return;
    }
    if (playMode.value === "shuffle") {
      const prev = Math.floor(Math.random() * queue.value.length);
      await playQueueIndex(prev, { smoothTransition: !manual, fade: !manual, prebuffer: manual });
      return;
    }
    const prevIndex = currentIndex.value - 1;
    if (prevIndex >= 0) {
      await playQueueIndex(prevIndex, { smoothTransition: !manual, fade: !manual, prebuffer: manual });
    } else if (playMode.value === "repeat-all") {
      await playQueueIndex(queue.value.length - 1, { smoothTransition: !manual, fade: !manual, prebuffer: manual });
    }
  };

  const initSpectrum = () => {
    if (!audioEl.value || typeof window === "undefined") return;
    if (!spectrumCtx) {
      spectrumCtx = new (window.AudioContext || window.webkitAudioContext)();
    }
    if (spectrumSource) return;
    spectrumAnalyser = spectrumCtx.createAnalyser();
    spectrumAnalyser.fftSize = 128;
    spectrumAnalyser.smoothingTimeConstant = 0.85;
    spectrumSource = spectrumCtx.createMediaElementSource(audioEl.value);
    spectrumSource.connect(spectrumAnalyser);
    spectrumAnalyser.connect(spectrumCtx.destination);
  };

  const startSpectrum = () => {
    if (!spectrumAnalyser || spectrumRaf) return;
    const buffer = new Uint8Array(spectrumAnalyser.frequencyBinCount);
    const tick = () => {
      spectrumAnalyser.getByteFrequencyData(buffer);
      const buckets = spectrum.value.length || 24;
      const step = Math.max(1, Math.floor(buffer.length / buckets));
      const next = [];
      for (let i = 0; i < buckets; i += 1) {
        let sum = 0;
        for (let j = 0; j < step; j += 1) {
          sum += buffer[i * step + j] || 0;
        }
        next.push(Math.min(1, sum / (step * 255)));
      }
      spectrum.value = next;
      spectrumRaf = window.requestAnimationFrame(tick);
    };
    spectrumRaf = window.requestAnimationFrame(tick);
  };

  const stopSpectrum = () => {
    if (spectrumRaf) {
      window.cancelAnimationFrame(spectrumRaf);
      spectrumRaf = 0;
    }
    if (spectrumDecayRaf) {
      window.cancelAnimationFrame(spectrumDecayRaf);
      spectrumDecayRaf = 0;
    }
    const step = () => {
      const next = spectrum.value.map((v) => Math.max(0.04, v * 0.82));
      spectrum.value = next;
      if (next.some((v) => v > 0.06)) {
        spectrumDecayRaf = window.requestAnimationFrame(step);
      } else {
        spectrum.value = Array.from({ length: spectrum.value.length || 24 }, () => 0.05);
        spectrumDecayRaf = 0;
      }
    };
    spectrumDecayRaf = window.requestAnimationFrame(step);
  };

  const fadeVolume = (from, to, durationMs = 280) => {
    if (!audioEl.value) return;
    if (volumeFadeRaf) {
      window.cancelAnimationFrame(volumeFadeRaf);
      volumeFadeRaf = 0;
    }
    const start = performance.now();
    const delta = to - from;
    const tick = (now) => {
      const t = Math.min(1, (now - start) / durationMs);
      const eased = t * (2 - t);
      const next = from + delta * eased;
      audioEl.value.volume = Math.max(0, Math.min(1, next));
      if (t < 1) {
        volumeFadeRaf = window.requestAnimationFrame(tick);
      } else {
        volumeFadeRaf = 0;
      }
    };
    volumeFadeRaf = window.requestAnimationFrame(tick);
  };

  const togglePlay = async () => {
    if (!audioEl.value) return;
    if (audioEl.value.paused) {
      if (!currentTrack.value && queue.value.length) {
        await playQueueIndex(currentIndex.value >= 0 ? currentIndex.value : 0);
        return;
      }
      if (!audioEl.value.src && queue.value.length) {
        await playQueueIndex(currentIndex.value >= 0 ? currentIndex.value : 0);
        return;
      }
      initSpectrum();
      if (spectrumCtx?.state === "suspended") {
        spectrumCtx.resume().catch(() => {});
      }
      audioEl.value.volume = 0;
      await audioEl.value.play().catch(() => {});
      isPlaying.value = true;
      startSpectrum();
      fadeVolume(0, volume.value, 320);
    } else {
      const currentVol = audioEl.value.volume ?? volume.value;
      fadeVolume(currentVol, 0, 260);
      setTimeout(() => {
        if (audioEl.value) audioEl.value.pause();
      }, 260);
      isPlaying.value = false;
      stopSpectrum();
    }
  };

  const startPlayback = async () => {
    if (!queue.value.length) {
      if (authToken.value) {
        await fetchAll(true);
      }
      if (!queue.value.length) {
        playError.value = "播放队列为空";
        return;
      }
    }
    if (currentIndex.value < 0) {
      currentIndex.value = 0;
    }
    await playQueueIndex(currentIndex.value);
  };

  const stopPlay = () => {
    if (!audioEl.value) return;
    const currentVol = audioEl.value.volume ?? volume.value;
    fadeVolume(currentVol, 0, 260);
    setTimeout(() => {
      if (audioEl.value) audioEl.value.pause();
    }, 260);
    audioEl.value.currentTime = 0;
    isPlaying.value = false;
    stopSpectrum();
  };

  const seekTo = (percent) => {
    if (!audioEl.value || !durationSeconds.value) return;
    const next = Math.max(0, Math.min(1, percent)) * durationSeconds.value;
    audioEl.value.currentTime = next;
  };

  const setVolume = (value) => {
    const next = Math.max(0, Math.min(1, value));
    volume.value = next;
    if (audioEl.value) {
      audioEl.value.volume = next;
    }
  };

  const setPlayMode = (mode) => {
    if (!mode) return;
    playMode.value = mode;
  };

  const recordPlayback = async (item) => {
    if (!item || !authToken.value) return;
    try {
      await fetch(apiUrl("/api/playback/events"), {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${authToken.value}`
        },
        body: JSON.stringify({
          track_id: item.id?.startsWith("netease-") ? undefined : item.id,
          source: item.source,
          source_id: item.source_id,
          title: item.title,
          artist: item.artist,
          duration_seconds: durationSeconds.value ? Math.round(durationSeconds.value) : undefined
        })
      });
    } catch {}
  };

  const cyclePlayMode = () => {
    const modes = ["order", "repeat-all", "repeat-one", "shuffle"];
    const idx = modes.indexOf(playMode.value);
    playMode.value = modes[(idx + 1) % modes.length];
  };

  const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

  const fadeOutAndPause = async (durationMs = 260) => {
    if (!audioEl.value) return;
    const currentVol = audioEl.value.volume ?? volume.value;
    fadeVolume(currentVol, 0, durationMs);
    await wait(durationMs);
    if (audioEl.value) audioEl.value.pause();
  };

  const startQrLogin = async () => {
    neteaseLoginError.value = "";
    qr.value = { key: "", qrimg: "", status: "loading", message: "" };
    qrErrorCount.value = 0;
    try {
      const keyResp = await apiGet("/api/netease/login/qr/key", { timestamp: Date.now() });
      const key = keyResp?.data?.unikey || keyResp?.unikey;
      if (!key) throw new Error(`获取二维码失败: ${JSON.stringify(keyResp)}`);
      const qrResp = await apiGet("/api/netease/login/qr/create", { key, platform: "web", qrimg: true, timestamp: Date.now() });
      qr.value = { key, qrimg: qrResp?.data?.qrimg || "", status: "waiting", message: "请扫码登录" };
      pollQrStatus();
    } catch (err) {
      qr.value = { key: "", qrimg: "", status: "error", message: err.message || "获取二维码失败" };
    }
  };

  const resumeIfPaused = () => {
    if (!audioEl.value) return;
    if (isPlaying.value && audioEl.value.paused) {
      audioEl.value.play().catch(() => {});
    }
  };

  const pollQrStatus = () => {
    if (!qr.value.key) return;
    qrPolling.value = true;
    clearInterval(qrTimer);
    qrTimer = setInterval(async () => {
      let res;
      const usedNoCookie = true;
      try {
        res = await apiGet("/api/netease/login/qr/check", {
          key: qr.value.key,
          timestamp: Date.now(),
          noCookie: "true"
        });
      } catch (err) {
        qrErrorCount.value += 1;
        qr.value = { ...qr.value, status: "error", message: err.message || "扫码失败，重试中..." };
        if (qrErrorCount.value >= 3) {
          qr.value = { ...qr.value, status: "error", message: "扫码失败，请稍后重试" };
          clearInterval(qrTimer);
          qrPolling.value = false;
        }
        return;
      }
      qrLast.value = {
        code: res?.code ?? null,
        at: new Date().toISOString(),
        noCookie: usedNoCookie,
        raw: res
      };
      const code = res?.code;
      if (code === 803) {
        qrErrorCount.value = 0;
        if (res?.cookie) {
          setNeteaseCookie(res.cookie);
        }
        await getLoginStatus(res?.cookie || neteaseCookie.value);
        qr.value = { ...qr.value, status: "success", message: "登录成功" };
        clearInterval(qrTimer);
        qrPolling.value = false;
      } else if (code === 800) {
        qrErrorCount.value = 0;
        qr.value = { ...qr.value, status: "expired", message: "二维码已过期" };
        clearInterval(qrTimer);
        qrPolling.value = false;
      } else if (code === 801) {
        qrErrorCount.value = 0;
        qr.value = { ...qr.value, status: "waiting", message: "等待扫码" };
      } else if (code === 802) {
        qrErrorCount.value = 0;
        qr.value = { ...qr.value, status: "scanned", message: "已扫码，等待确认" };
      }
    }, 3000);
  };

  const neteaseLogout = async () => {
    try {
      await apiGet("/api/netease/logout", {
        cookie: buildNeteaseCookie() || undefined,
        timestamp: Date.now()
      });
    } catch {}
    neteaseCookie.value = "";
    neteaseProfile.value = null;
    if (typeof window !== "undefined") {
      window.localStorage.removeItem(getNeteaseCookieKey());
    }
  };

  const handleTimeUpdate = () => {
    if (!audioEl.value) return;
    currentSeconds.value = audioEl.value.currentTime || 0;
    durationSeconds.value = audioEl.value.duration || 0;
    if (lyricLines.value.length) {
      const t = currentSeconds.value;
      let idx = -1;
      if (t < 1 && lyricLines.value[0]?.time === 0) {
        idx = 0;
      } else {
        for (let i = 0; i < lyricLines.value.length; i += 1) {
          if (t >= lyricLines.value[i].time) idx = i;
          else break;
        }
      }
      currentLyricIndex.value = idx;
    }
    persistPlayerState();
    maybeSaveDuration();
    if (isPlaying.value && durationSeconds.value > 0 && durationSeconds.value - currentSeconds.value <= 12) {
      prefetchNextTrack();
    }
  };

  const maybeSaveDuration = async () => {
    if (!currentTrack.value || !durationSeconds.value) return;
    const trackId = currentTrack.value.id;
    if (!trackId) return;
    if (durationSaved.value[trackId]) return;
    durationSaved.value[trackId] = true;
    try {
      await fetch(apiUrl(`/api/tracks/${trackId}`), {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${authToken.value}`
        },
        body: JSON.stringify({
          duration_seconds: Math.round(durationSeconds.value)
        })
      });
    } catch {}
  };

  const loadNeteaseQuality = async (item) => {
    if (!item || item.source !== "netease") return;
    try {
      const data = await apiGet("/api/netease/song/url/v1", {
        id: item.source_id || item.id,
        level: neteaseQualityLevel.value || "exhigh",
        unblock: "true",
        cookie: buildNeteaseCookie() || undefined
      });
      const info = data?.data?.[0];
      if (info) {
        neteaseQualityMeta.value = {
          level: info.level || neteaseQualityLevel.value,
          type: info.type || "",
          br: info.br || null
        };
      }
    } catch {}
  };

  const qualityText = computed(() => {
    if (currentTrack.value?.source === "netease") {
      const meta = neteaseQualityMeta.value;
      const level = meta?.level || neteaseQualityLevel.value || "";
      const type = meta?.type ? meta.type.toUpperCase() : "";
      const br = meta?.br ? `${Math.round(meta.br / 1000)}kbps` : "";
      const levelDisplay = level ? level.toString().toUpperCase() : "";
      return [levelDisplay, type, br].filter(Boolean).join(" · ");
    }
    return currentTrack.value?.quality || nowPlaying.value?.quality || "";
  });

  const persistPlayerState = () => {
    if (typeof window === "undefined") return;
    const now = Date.now();
    if (now - lastPersistAt.value < 2000) return;
    lastPersistAt.value = now;
    const payload = {
      track: currentTrack.value,
      index: currentIndex.value,
      seconds: currentSeconds.value,
      volume: volume.value,
      playMode: playMode.value
    };
    window.localStorage.setItem("player-state", JSON.stringify(payload));
  };

  const handlePlay = () => {
    isPlaying.value = true;
  };

  const handlePause = () => {
    isPlaying.value = false;
  };

  const handleEnded = () => {
    playNext(false);
  };

  const handleError = () => {
    const code = audioEl.value?.error?.code;
    const hint = code ? `播放错误 (code=${code})` : "播放错误";
    playError.value = hint;
    isPlaying.value = false;
  };

  watch(audioEl, (el, prev) => {
    if (prev) {
      prev.removeEventListener("timeupdate", handleTimeUpdate);
      prev.removeEventListener("loadedmetadata", handleTimeUpdate);
      prev.removeEventListener("play", handlePlay);
      prev.removeEventListener("pause", handlePause);
      prev.removeEventListener("ended", handleEnded);
      prev.removeEventListener("error", handleError);
    }
    if (el) {
      el.addEventListener("timeupdate", handleTimeUpdate);
      el.addEventListener("loadedmetadata", handleTimeUpdate);
      el.addEventListener("play", handlePlay);
      el.addEventListener("pause", handlePause);
      el.addEventListener("ended", handleEnded);
      el.addEventListener("error", handleError);
      el.preload = "auto";
      el.crossOrigin = "anonymous";
      el.volume = volume.value;
    }
  });

  watch(volume, () => {
    persistPlayerState();
  });

  watch(playMode, () => {
    persistPlayerState();
  });

  watch(currentTrack, () => {
    persistPlayerState();
  });

  const getLoginStatus = async (cookie = "") => {
    try {
      const res = await fetch(apiUrl("/api/netease/login/status", { timestamp: Date.now() }), {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${authToken.value}`
        },
        body: JSON.stringify({ cookie })
      });
      if (res.ok) {
        const data = await res.json();
        neteaseProfile.value = data?.data?.profile || data?.profile || null;
        qrLast.value = {
          ...qrLast.value,
          raw: data
        };
      }
    } catch {}
  };

  let timerId;

  onMounted(() => {
    const stored = typeof window !== "undefined" ? window.localStorage.getItem("music-theme") : null;
    if (stored === "night" || stored === "day") {
      applyTheme(stored === "night");
    } else {
      const hour = new Date().getHours();
      applyTheme(hour >= 19 || hour < 7);
    }
    updateTime();
    timerId = window.setInterval(updateTime, 1000);
    if (authToken.value) {
      fetchAll(true);
    }
    if (neteaseCookie.value) {
      apiGet("/api/netease/user/account", { cookie: buildNeteaseCookie(), timestamp: Date.now() })
        .then((data) => {
          neteaseProfile.value = data?.profile || null;
        })
        .catch(() => {});
    }
    if (currentTrack.value?.source === "netease") {
      preloadLyricForItem(currentTrack.value);
    }
    restoreLyricCache();
    if (typeof window !== "undefined") {
      const saved = window.localStorage.getItem("player-state");
      if (saved) {
        try {
          const parsed = JSON.parse(saved);
          savedPlayback.value = parsed || null;
          if (parsed?.track) currentTrack.value = parsed.track;
          if (typeof parsed?.index === "number") currentIndex.value = parsed.index;
          if (typeof parsed?.seconds === "number") currentSeconds.value = parsed.seconds;
          if (typeof parsed?.track?.duration_seconds === "number") {
            durationSeconds.value = parsed.track.duration_seconds;
          }
          if (typeof parsed?.volume === "number") {
            volume.value = Math.max(0, Math.min(1, parsed.volume));
            if (audioEl.value) audioEl.value.volume = volume.value;
          }
          if (parsed?.playMode) playMode.value = parsed.playMode;
        } catch {}
      }
    }
    if (currentTrack.value) {
      const cacheKey = currentTrack.value.source_id || currentTrack.value.id;
      const cached = cacheKey ? lyricCache.value[cacheKey] : null;
      if (cached?.lines?.length) {
        currentLyric.value = cached.raw || "";
        lyricLines.value = cached.lines;
      }
      prepareTrackForResume(currentTrack.value, currentSeconds.value);
    }
  });

  onBeforeUnmount(() => {
    if (timerId) window.clearInterval(timerId);
    clearInterval(qrTimer);
  });

  return {
    apiBase,
    apiReady,
    apiBaseValid,
    isNight,
    time,
    date,
    timeProgress,
    currentTimeText,
    durationText,
    nextUpText,
    toggleTheme,
    authToken,
    isAuthed,
    authLoading,
    authError,
    authUsername,
    authIsAdmin,
    isAdmin,
    resolveCoverUrl,
    loginForm,
    login,
    logout,
    userCreateForm,
    userCreateLoading,
    userCreateError,
    userCreateSuccess,
    createUser,
    passwordForm,
    passwordLoading,
    passwordError,
    passwordSuccess,
    changePassword,
    adminResetForm,
    adminResetLoading,
    adminResetError,
    adminResetSuccess,
    adminResetPassword,
    dataLoading,
    dataError,
    lastFetchedAt,
    fetchAll,
    nowPlaying,
    queue,
    playlists,
    library,
    devices,
    deviceCreateForm,
    deviceCreateLoading,
    deviceCreateError,
    deviceCreateResult,
    createDevice,
    syncDevice,
    deleteDevice,
    stats,
    uploadFile,
    uploadLyricFile,
    uploadCoverFile,
    uploadForm,
    uploadLoading,
    uploadError,
    uploadSuccess,
    onFileChange,
    onLyricFileChange,
    onCoverFileChange,
    uploadMusic,
    queueReplaceWithPlaylist,
    queueClearAll,
    queueAddItems,
    queueReorder,
    addTrackToPlaylist,
    createPlaylist,
    deletePlaylist,
    removeTrackFromPlaylist,
    deleteTrack,
    fetchPlaylistDetail,
    searchQuery,
    searchLoading,
    searchError,
    searchResults,
    selectedPlaylistId,
    searchNetease,
    neteaseImport,
    importNeteasePlaylist,
    playlistDetail,
    playlistDetailLoading,
    playlistDetailError,
    playlistDetailDurationPending,
    playlistCreateForm,
    playlistCreateLoading,
    playlistCreateError,
    playFromNetease,
    audioEl,
    isPlaying,
    isBuffering,
    currentIndex,
    durationSeconds,
    currentSeconds,
    volume,
    playMode,
    playError,
    currentTrack,
    currentLyric,
    currentLyricLoading,
    currentLyricError,
    lyricLines,
    currentLyricIndex,
    lyricCache,
    neteaseQualityMeta,
    queueDockOpen,
    uploadDockOpen,
    playbackTrend,
    formatRelativeTime,
    spectrum,
    qualityText,
    preloadLyricBatch,
    playQueueIndex,
    playNext,
    playPrev,
    togglePlay,
    startPlayback,
    stopPlay,
    seekTo,
    setVolume,
    setPlayMode,
    cyclePlayMode,
    neteaseCookie,
    neteaseCookieMode,
    setNeteaseCookieMode,
    neteaseProfile,
    neteaseLoginLoading,
    neteaseLoginError,
    neteasePrompt,
    dismissNeteasePrompt,
    neteaseLogout,
    qr,
    qrPolling,
    qrLast,
    getLoginStatus,
    resumeIfPaused,
    startQrLogin
  };
};

export const useMusicState = () => {
  if (!sharedState) {
    sharedState = createState();
  }
  return sharedState;
};
