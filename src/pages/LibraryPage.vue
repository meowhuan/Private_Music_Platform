<script setup>
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useMusicState } from "../composables/useMusicState";
defineOptions({ name: "LibraryPage" });

const state = useMusicState();
const playlistOpen = ref(false);
const selectedPlaylistLabel = computed(() => {
  const current = state.playlists.value.find((item) => item.id === state.selectedPlaylistId.value);
  return current?.name || "导入为新歌单";
});

const togglePlaylist = () => {
  playlistOpen.value = !playlistOpen.value;
};

const pickPlaylist = (id) => {
  state.selectedPlaylistId.value = id;
  playlistOpen.value = false;
};

const handleDocClick = (event) => {
  const target = event.target;
  if (!target || !(target instanceof Element)) return;
  if (!target.closest(".playlist-select")) {
    playlistOpen.value = false;
  }
};

onMounted(() => {
  document.addEventListener("click", handleDocClick);
});

onBeforeUnmount(() => {
  document.removeEventListener("click", handleDocClick);
});
</script>

<template>
  <section id="library" class="mt-12">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <h2 class="font-display text-2xl">最近曲库</h2>
      <div class="flex flex-wrap items-center gap-2">
        <div class="playlist-select">
          <button class="playlist-select-btn" type="button" @click="togglePlaylist">
            <span class="truncate">{{ selectedPlaylistLabel }}</span>
            <span class="playlist-select-arrow" :class="playlistOpen ? 'playlist-select-arrow-open' : ''">▾</span>
          </button>
          <transition name="select-pop">
            <div v-if="playlistOpen" class="playlist-select-menu">
              <button class="playlist-select-item" type="button" @click="pickPlaylist('')">导入为新歌单</button>
              <button
                v-for="plist in state.playlists.value"
                :key="plist.id"
                class="playlist-select-item"
                type="button"
                @click="pickPlaylist(plist.id)"
              >
                {{ plist.name }}
              </button>
            </div>
          </transition>
        </div>
        <span class="meow-pill">{{ state.dataLoading.value ? "更新中" : "已同步" }}</span>
        <button
          class="meow-pill motion-press"
          type="button"
          @click="state.uploadDockOpen.value = true"
        >
          上传音乐
        </button>
      </div>
    </div>
    <div class="mt-6 space-y-3 library-scroll">
      <div v-if="state.dataLoading.value && !state.library.value.length" class="meow-card motion-card p-5 skeleton-card" :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''"></div>
      <div v-if="state.dataLoading.value && !state.library.value.length" class="meow-card motion-card p-5 skeleton-card" :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''"></div>
      <div v-if="state.dataLoading.value && !state.library.value.length" class="meow-card motion-card p-5 skeleton-card" :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''"></div>
      <article
        v-for="(track, idx) in state.library.value"
        :key="track.id || track.title"
        class="meow-card motion-card library-card stagger-card"
        :style="{ '--stagger': `${0.04 * (idx + 1)}s` }"
        :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''"
      >
        <div class="library-cover" :class="state.isNight.value ? 'library-cover-night' : 'library-cover-day'">
          <img v-if="track.cover_url" :src="state.resolveCoverUrl(track.cover_url)" alt="cover" />
        </div>
        <div class="library-meta">
          <div class="font-600">{{ track.title }}</div>
          <div class="text-sm" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
            {{ track.artist }} <span class="opacity-70">·</span> {{ track.album || "未知专辑" }}
          </div>
          <div class="library-badges">
            <span class="meow-pill">{{ track.tag || "离线" }}</span>
            <span class="text-xs" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">{{ track.time }}</span>
          </div>
        </div>
        <div class="library-actions">
          <button class="meow-pill motion-press" type="button" @click="state.queueAddItems([{ track_id: track.id, source: track.source }])">加入队列</button>
          <button class="meow-pill motion-press" type="button" @click="state.addTrackToPlaylist(state.selectedPlaylistId.value, track)" :disabled="!state.selectedPlaylistId.value">加入歌单</button>
          <button class="meow-pill motion-press" type="button" @click="state.deleteTrack(track.id)">删除</button>
        </div>
      </article>
      <div v-if="!state.library.value.length && !state.dataLoading.value" class="mt-4 text-sm" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
        暂无曲库数据
      </div>
    </div>
  </section>

  <section id="upload" class="mt-16"></section>
</template>
