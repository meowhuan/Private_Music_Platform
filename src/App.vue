<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useRoute } from "vue-router";
import { useMusicState } from "./composables/useMusicState";

const state = useMusicState();
const route = useRoute();
const showSearch = ref(false);
const searchPlaylistOpen = ref(false);
const searchInputEl = ref(null);
const showIntro = ref(false);
const isDragOver = ref(false);
const isCoverDragOver = ref(false);
const queueQuery = ref("");
const queueListEl = ref(null);
const coverPreviewUrl = ref("");

const openSearch = async () => {
  showSearch.value = true;
  await nextTick();
  searchInputEl.value?.focus();
};

const closeSearch = () => {
  showSearch.value = false;
};

const toggleSearch = () => {
  if (showSearch.value) {
    closeSearch();
  } else {
    openSearch();
  }
};

const isInputActive = (target) => {
  const tag = target?.tagName?.toLowerCase();
  return tag === "input" || tag === "textarea" || target?.isContentEditable;
};

const onKeydown = (event) => {
  if (isInputActive(event.target)) return;
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
    event.preventDefault();
    toggleSearch();
    return;
  }
  if (event.key === "/") {
    event.preventDefault();
    openSearch();
  }
  if (event.key === "Escape" && showSearch.value) {
    closeSearch();
  }
};

const selectedSearchPlaylistLabel = computed(() => {
  const current = state.playlists.value.find((item) => item.id === state.selectedPlaylistId.value);
  return current?.name || "选择歌单";
});

const toggleSearchPlaylist = () => {
  searchPlaylistOpen.value = !searchPlaylistOpen.value;
};

const pickSearchPlaylist = (id) => {
  state.selectedPlaylistId.value = id;
  searchPlaylistOpen.value = false;
};

const handleDocClick = (event) => {
  const target = event.target;
  if (!target || !(target instanceof Element)) return;
  if (!target.closest(".playlist-select")) {
    searchPlaylistOpen.value = false;
  }
};

onMounted(() => {
  window.addEventListener("keydown", onKeydown);
  document.addEventListener("click", handleDocClick);
  const hasSeenIntro = typeof window !== "undefined" && window.localStorage.getItem("music-intro-seen") === "1";
  if (!hasSeenIntro) {
    showIntro.value = true;
    window.setTimeout(() => {
      showIntro.value = false;
      window.localStorage.setItem("music-intro-seen", "1");
    }, 520);
  }
});

watch(
  () => route.path,
  () => {
    state.resumeIfPaused();
  }
);

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeydown);
  document.removeEventListener("click", handleDocClick);
  if (coverPreviewUrl.value) {
    URL.revokeObjectURL(coverPreviewUrl.value);
    coverPreviewUrl.value = "";
  }
});

const navLinks = [
  { to: "/", label: "主页" },
  { to: "/library", label: "曲库" },
  { to: "/playlists", label: "歌单" },
  { to: "/devices", label: "设备" },
  { to: "/settings", label: "设置" }
];

const isHome = computed(() => route.path === "/");
const themeClass = computed(() =>
  state.isNight.value
    ? "bg-gradient-to-br from-meow-night-bg via-[#201a3f] to-[#16162a] text-meow-night-ink meow-night"
    : "bg-gradient-to-br from-meow-bg via-[#fff6fb] to-[#f2f0ff] text-meow-ink meow-day"
);

const uploadFilename = computed(() => state.uploadFile.value?.name || "未选择文件");

const updateCoverPreview = (file) => {
  if (coverPreviewUrl.value) {
    URL.revokeObjectURL(coverPreviewUrl.value);
    coverPreviewUrl.value = "";
  }
  if (file) {
    coverPreviewUrl.value = URL.createObjectURL(file);
  }
};

const onCoverDrop = (event) => {
  const file = event?.dataTransfer?.files?.[0];
  state.uploadCoverFile.value = file || null;
  updateCoverPreview(file || null);
};

watch(
  () => state.uploadCoverFile.value,
  (file) => {
    updateCoverPreview(file || null);
  }
);

const filteredQueue = computed(() => {
  const query = queueQuery.value.trim().toLowerCase();
  const source = state.queue.value.map((item, index) => ({ item, index }));
  if (!query) return source;
  return source.filter(({ item }) => {
    const title = (item.title || "").toLowerCase();
    const artist = (item.artist || "").toLowerCase();
    return title.includes(query) || artist.includes(query);
  });
});

const scrollToCurrent = async () => {
  await nextTick();
  const wrap = queueListEl.value;
  if (!wrap) return;
  const active = wrap.querySelector(".dock-queue-item-active");
  if (!active) return;
  const offset = active.offsetTop - wrap.clientHeight * 0.75;
  wrap.scrollTo({ top: Math.max(0, offset), behavior: "smooth" });
};
</script>

<template>
  <div class="min-h-screen font-body page-fade transition-colors duration-700 ease-in-out meow-bg" :class="themeClass">
    <div class="relative overflow-hidden">
      <div
        class="pointer-events-none absolute -left-32 -top-24 h-80 w-80 rounded-[45%_55%_60%_40%/50%_60%_40%_50%] blur-3xl opacity-70 animate-floaty"
        :class="state.isNight.value
          ? 'bg-[radial-gradient(circle_at_top,_#3a2b6f,_transparent_65%)]'
          : 'bg-[radial-gradient(circle_at_top,_#ffd4e6,_transparent_65%)]'"
      ></div>
      <div
        class="pointer-events-none absolute -right-32 top-24 h-96 w-96 rounded-[55%_45%_45%_55%/45%_55%_45%_55%] blur-3xl opacity-70 animate-floaty"
        :class="state.isNight.value
          ? 'bg-[radial-gradient(circle_at_top,_#1d5c7a,_transparent_65%)]'
          : 'bg-[radial-gradient(circle_at_top,_#c8f6ed,_transparent_65%)]'"
      ></div>

      <div class="mx-auto w-[min(1100px,92vw)] pb-20 pt-8 relative">
        <button
          class="cord-switch cord-switch-mobile md:hidden"
          type="button"
          @click="state.toggleTheme"
          :class="state.isNight.value ? 'cord-switch-night' : 'cord-switch-day'"
          aria-label="切换深夜模式"
        >
          <span class="cord-line"></span>
          <span class="cord-knob">{{ state.isNight.value ? "🌙" : "☀️" }}</span>
          <span class="cord-label" aria-hidden="true"></span>
        </button>

        <nav class="flex flex-wrap items-center justify-between gap-4">
          <div class="flex items-center gap-3">
            <img
              src="/logo.svg"
              alt="Music logo"
              class="h-10 w-10 rounded-full border bg-white/70 object-cover shadow-sm"
              :class="state.isNight.value ? 'border-meow-night-line' : 'border-meow-line'"
            />
            <div class="font-display text-xl tracking-wide">Meow Music</div>
          </div>
          <div class="nav-links-wrap hidden items-center justify-end gap-5 text-sm md:flex" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
            <button
              class="cord-switch cord-switch-desktop cord-switch-desktop-left"
              type="button"
              @click="state.toggleTheme"
              :class="state.isNight.value ? 'cord-switch-night' : 'cord-switch-day'"
              aria-label="切换深夜模式"
            >
              <span class="cord-line"></span>
              <span class="cord-knob">{{ state.isNight.value ? "🌙" : "☀️" }}</span>
              <span class="cord-label" aria-hidden="true"></span>
            </button>
            <router-link
              v-for="link in navLinks"
              :key="link.to"
              class="nav-link"
              :class="state.isNight.value ? 'hover:text-meow-night-ink' : 'hover:text-meow-ink'"
              :to="link.to"
            >
              {{ link.label }}
            </router-link>
            <button class="meow-pill motion-press" type="button" @click="openSearch">搜索 / Ctrl+K</button>
            <button
              class="cord-switch cord-switch-desktop cord-switch-desktop-right"
              type="button"
              @click="state.toggleTheme"
              :class="state.isNight.value ? 'cord-switch-night' : 'cord-switch-day'"
              aria-label="切换深夜模式"
            >
              <span class="cord-line"></span>
              <span class="cord-knob">{{ state.isNight.value ? "🌙" : "☀️" }}</span>
              <span class="cord-label" aria-hidden="true"></span>
            </button>
          </div>
        </nav>
        <div class="mobile-nav md:hidden" :class="state.isNight.value ? 'mobile-nav-night' : ''">
          <div class="mobile-nav-links">
            <router-link
              v-for="link in navLinks"
              :key="link.to"
              class="mobile-nav-link"
              :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'"
              :to="link.to"
            >
              {{ link.label }}
            </router-link>
          </div>
          <button class="meow-pill motion-press" type="button" @click="openSearch">🔍</button>
        </div>

        <router-view v-slot="{ Component, route }">
          <transition name="page-slide" mode="out-in">
            <keep-alive include="HomePage,LibraryPage,PlaylistsPage,DevicesPage,SettingsPage">
              <component :is="Component" :key="route.fullPath" />
            </keep-alive>
          </transition>
        </router-view>

        <footer class="mt-16 text-center text-xs" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
          © 2026 Meow Music. 私人音乐平台 · 仅供自用
        </footer>
      </div>
    </div>

    <div v-if="showSearch" class="modal-mask" @click.self="closeSearch">
      <div class="modal-card" :class="state.isNight.value ? 'modal-card-night' : ''">
        <div class="flex items-center justify-between gap-3">
          <div>
            <div class="text-[11px] uppercase tracking-widest" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">网易云搜索</div>
            <div class="mt-1 text-sm" :class="state.isNight.value ? 'text-meow-night-ink' : 'text-meow-ink'">
              快捷键：Ctrl+K 或 /
            </div>
          </div>
          <button class="meow-pill motion-press" type="button" @click="closeSearch">关闭</button>
        </div>
        <div class="mt-4 flex flex-wrap gap-3">
          <input
            ref="searchInputEl"
            v-model="state.searchQuery.value"
            class="meow-input"
            :class="state.isNight.value ? 'meow-input-night' : ''"
            type="text"
            placeholder="输入关键词"
            @keydown.enter.prevent="state.searchNetease"
          />
          <div class="playlist-select">
            <button class="playlist-select-btn" type="button" @click="toggleSearchPlaylist">
              <span class="truncate">{{ selectedSearchPlaylistLabel }}</span>
              <span class="playlist-select-arrow" :class="searchPlaylistOpen ? 'playlist-select-arrow-open' : ''">▾</span>
            </button>
            <transition name="select-pop">
              <div v-if="searchPlaylistOpen" class="playlist-select-menu">
                <button class="playlist-select-item" type="button" @click="pickSearchPlaylist('')">选择歌单</button>
                <button
                  v-for="plist in state.playlists.value"
                  :key="plist.id"
                  class="playlist-select-item"
                  type="button"
                  @click="pickSearchPlaylist(plist.id)"
                >
                  {{ plist.name }}
                </button>
              </div>
            </transition>
          </div>
          <button
            class="meow-btn-primary motion-press"
            :class="state.isNight.value ? 'bg-meow-night-accent text-meow-night-bg' : ''"
            type="button"
            @click="state.searchNetease"
            :disabled="state.searchLoading.value"
          >
            {{ state.searchLoading.value ? "搜索中" : "搜索" }}
          </button>
        </div>
        <div v-if="state.searchError.value" class="mt-2 text-xs text-[color:#e4547a]">{{ state.searchError.value }}</div>
        <div class="mt-4 grid gap-3">
          <div
            v-for="track in state.searchResults.value"
            :key="track.id"
            class="meow-card motion-card search-row p-4"
            :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''"
          >
            <div>
              <div class="font-600">{{ track.name }}</div>
              <div class="mt-1 text-xs" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
                {{ track.artists }} · {{ track.album }}
              </div>
            </div>
            <div class="flex items-center gap-2">
              <button class="meow-pill motion-press" type="button" @click="state.playFromNetease(track)">播放</button>
              <button
                class="meow-pill motion-press"
                type="button"
                @click="state.queueAddItems([{ source: 'netease', source_id: String(track.id), title: track.name, artist: track.artists, album: track.album, cover_url: track.cover_url }])"
              >
                加入队列
              </button>
              <button
                class="meow-pill motion-press"
                type="button"
                @click="state.addTrackToPlaylist(state.selectedPlaylistId.value, { source: 'netease', source_id: String(track.id), title: track.name, artist: track.artists, album: track.album, cover_url: track.cover_url })"
                :disabled="!state.selectedPlaylistId.value"
              >
                加入歌单
              </button>
            </div>
          </div>
          <div v-if="!state.searchResults.value.length && !state.searchLoading.value" class="text-xs" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
            暂无搜索结果
          </div>
        </div>
      </div>
    </div>

    <audio :ref="state.audioEl" class="sr-only"></audio>
  </div>

  <div v-if="state.queueDockOpen.value || state.uploadDockOpen.value" class="dock-mask" @click="state.queueDockOpen.value = false; state.uploadDockOpen.value = false"></div>
  <div class="dock-panel" :class="[state.queueDockOpen.value ? 'dock-panel-open' : '', state.isNight.value ? 'dock-panel-night' : '']">
    <div class="dock-head">
      <div class="dock-head-left">
        <div class="font-600">播放队列</div>
        <span class="dock-count" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
          共 {{ state.queue.value.length }} 首 · 显示 {{ filteredQueue.length }} 首
        </span>
      </div>
      <div class="dock-head-actions">
        <button class="meow-pill motion-press" type="button" @click="scrollToCurrent">定位当前</button>
        <button class="meow-pill motion-press" type="button" @click="state.queueDockOpen.value = false">关闭</button>
      </div>
    </div>
    <div class="dock-now">
      <div class="dock-now-meta">
        <div class="dock-now-title">{{ state.currentTrack.value?.title || "未播放" }}</div>
        <div class="dock-now-sub" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
          {{ state.currentTrack.value?.artist || "" }}
        </div>
      </div>
      <div class="dock-progress">
        <div class="music-progress" @click="state.seekTo(($event.offsetX || 0) / $event.currentTarget.clientWidth)">
          <div class="music-progress-fill" :style="{ width: state.timeProgress.value }"></div>
        </div>
        <div class="dock-time" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
          <span>{{ state.currentTimeText.value }}</span>
          <span>{{ state.durationText.value }}</span>
        </div>
      </div>
    </div>
    <div class="dock-search">
      <input
        v-model="queueQuery"
        type="text"
        class="dock-search-input"
        placeholder="搜索队列歌曲/歌手"
      />
    </div>
    <div ref="queueListEl" class="dock-queue">
      <div v-if="!filteredQueue.length" class="text-xs" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
        队列为空
      </div>
      <div
        v-for="entry in filteredQueue"
        :key="entry.item.queue_id || entry.item.id || entry.item.title"
        class="dock-queue-item"
        :class="state.currentIndex.value === entry.index ? 'dock-queue-item-active' : ''"
        @click="state.playQueueIndex(entry.index)"
      >
        <div class="dock-queue-meta">
          <div class="truncate">{{ entry.item.title }}</div>
          <div class="text-[11px] opacity-70 truncate">{{ entry.item.artist }}</div>
        </div>
        <div class="dock-queue-time">{{ entry.item.time || "00:00" }}</div>
      </div>
    </div>
    <div
      v-if="state.nextUpText.value"
      class="text-xs"
      :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'"
    >
      {{ state.nextUpText.value }}
    </div>
  </div>

  <div class="dock-panel upload-dock" :class="[state.uploadDockOpen.value ? 'dock-panel-open' : '', state.isNight.value ? 'dock-panel-night' : '']">
    <div class="dock-head">
      <div class="font-600">上传音乐</div>
      <button class="meow-pill motion-press" type="button" @click="state.uploadDockOpen.value = false">关闭</button>
    </div>
    <div class="upload-dock-form">
      <input
        class="meow-input"
        :class="state.isNight.value ? 'meow-input-night' : ''"
        type="text"
        placeholder="曲目名称（可选）"
        v-model="state.uploadForm.value.title"
      />
      <input
        class="meow-input"
        :class="state.isNight.value ? 'meow-input-night' : ''"
        type="text"
        placeholder="歌手（可选）"
        v-model="state.uploadForm.value.artist"
      />
      <input
        class="meow-input"
        :class="state.isNight.value ? 'meow-input-night' : ''"
        type="text"
        placeholder="专辑（可选）"
        v-model="state.uploadForm.value.album"
      />
      <input
        class="meow-input"
        :class="state.isNight.value ? 'meow-input-night' : ''"
        type="text"
        placeholder="标签（默认 离线）"
        v-model="state.uploadForm.value.tag"
      />
      <div
        class="file-picker file-picker-drop"
        :class="[{ 'file-picker-active': isDragOver }, state.isNight.value ? 'file-picker-night' : '']"
        @dragover.prevent="isDragOver = true"
        @dragleave.prevent="isDragOver = false"
        @drop.prevent="isDragOver = false; state.onFileChange($event)"
      >
        <label class="file-picker-btn">
          选择文件
          <input class="file-input-hidden" type="file" multiple accept="audio/*,.lrc,.txt" @change="state.onFileChange" />
        </label>
        <span class="file-picker-name">{{ uploadFilename }}</span>
        <span class="file-picker-hint">拖拽到此处</span>
      </div>
      <div class="file-picker" :class="state.isNight.value ? 'file-picker-night' : ''">
        <label class="file-picker-btn">
          选择歌词
          <input class="file-input-hidden" type="file" accept=".lrc,.txt" @change="state.onLyricFileChange" />
        </label>
        <span class="file-picker-name">{{ state.uploadLyricFile.value?.name || "未选择歌词" }}</span>
      </div>
      <div
        class="file-picker file-picker-drop"
        :class="[{ 'file-picker-active': isCoverDragOver }, state.isNight.value ? 'file-picker-night' : '']"
        @dragover.prevent="isCoverDragOver = true"
        @dragleave.prevent="isCoverDragOver = false"
        @drop.prevent="isCoverDragOver = false; onCoverDrop($event)"
      >
        <label class="file-picker-btn">
          选择封面
          <input class="file-input-hidden" type="file" accept="image/*" @change="state.onCoverFileChange" />
        </label>
        <span class="file-picker-name">{{ state.uploadCoverFile.value?.name || "未选择封面" }}</span>
        <span class="file-picker-hint">拖拽到此处</span>
      </div>
      <div v-if="coverPreviewUrl" class="cover-preview" :class="state.isNight.value ? 'cover-preview-night' : ''">
        <img :src="coverPreviewUrl" alt="cover-preview" />
      </div>
      <button
        class="meow-btn-primary motion-press"
        :class="state.isNight.value ? 'bg-meow-night-accent text-meow-night-bg' : ''"
        type="button"
        @click="state.uploadMusic"
        :disabled="state.uploadLoading.value || !state.isAuthed.value"
      >
        {{ state.uploadLoading.value ? "上传中" : "上传" }}
      </button>
      <div v-if="!state.isAuthed.value" class="text-xs" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
        登录后可上传
      </div>
      <div v-if="state.uploadError.value" class="text-xs text-[color:#e4547a]">{{ state.uploadError.value }}</div>
      <div v-if="state.uploadSuccess.value" class="text-xs text-[color:#3aa889]">{{ state.uploadSuccess.value }}</div>
    </div>
  </div>

  <div v-if="state.neteasePrompt.value.show" class="netease-toast" :class="state.isNight.value ? 'netease-toast-night' : ''">
    <div class="text-sm font-600">网易云登录提示</div>
    <div class="mt-1 text-xs">{{ state.neteasePrompt.value.message }}</div>
    <div class="mt-1 text-xs" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">{{ state.neteasePrompt.value.action }}</div>
    <button class="meow-pill motion-press mt-2" type="button" @click="state.dismissNeteasePrompt">知道了</button>
  </div>

  <transition name="intro-fade">
    <div v-if="showIntro" class="intro-loader" :class="state.isNight.value ? 'intro-loader-night' : 'intro-loader-day'" aria-hidden="true">
      <div class="intro-loader-inner">
        <span class="intro-dot"></span>
        <span class="intro-dot"></span>
        <span class="intro-dot"></span>
      </div>
    </div>
  </transition>
</template>

<style src="./styles/music.css"></style>
