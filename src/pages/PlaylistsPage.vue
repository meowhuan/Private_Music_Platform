<script setup>
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useMusicState } from "../composables/useMusicState";
defineOptions({ name: "PlaylistsPage" });

const state = useMusicState();
const importOpen = ref(false);
const selectedImportLabel = computed(() => {
  const current = state.playlists.value.find((item) => item.id === state.neteaseImport.value.targetPlaylistId);
  return current?.name || "导入为新歌单";
});

const toggleImport = () => {
  importOpen.value = !importOpen.value;
};

const pickImport = (id) => {
  state.neteaseImport.value.targetPlaylistId = id;
  importOpen.value = false;
};

const handleDocClick = (event) => {
  const target = event.target;
  if (!target || !(target instanceof Element)) return;
  if (!target.closest(".playlist-select")) {
    importOpen.value = false;
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
  <section id="playlists" class="mt-12">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <h2 class="font-display text-2xl">精选歌单</h2>
      <router-link class="meow-pill motion-press" :class="state.isNight.value ? 'border-meow-night-line bg-meow-night-bg text-meow-night-ink' : ''" to="/library">查看曲库</router-link>
    </div>

    <div class="mt-6 grid gap-4 md:grid-cols-2">
      <div class="meow-card motion-card p-5 stagger-card" :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''" :style="{ '--stagger': '0.08s' }">
        <div class="text-xs uppercase tracking-widest" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">新建歌单</div>
        <div class="mt-3 grid gap-3">
          <input
            class="meow-input"
            :class="state.isNight.value ? 'meow-input-night' : ''"
            type="text"
            placeholder="歌单名称"
            v-model="state.playlistCreateForm.value.name"
          />
          <input
            class="meow-input"
            :class="state.isNight.value ? 'meow-input-night' : ''"
            type="text"
            placeholder="描述（可选）"
            v-model="state.playlistCreateForm.value.desc"
          />
          <input
            class="meow-input"
            :class="state.isNight.value ? 'meow-input-night' : ''"
            type="text"
            placeholder="标签 / 心情（可选）"
            v-model="state.playlistCreateForm.value.mood"
          />
          <button
            class="meow-btn-primary motion-press"
            :class="state.isNight.value ? 'bg-meow-night-accent text-meow-night-bg' : ''"
            type="button"
            @click="state.createPlaylist"
            :disabled="state.playlistCreateLoading.value"
          >
            {{ state.playlistCreateLoading.value ? "创建中" : "创建歌单" }}
          </button>
          <div v-if="state.playlistCreateError.value" class="text-xs text-[color:#e4547a]">{{ state.playlistCreateError.value }}</div>
        </div>
      </div>

      <div class="meow-card motion-card p-5 stagger-card" :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''" :style="{ '--stagger': '0.16s' }">
        <div class="text-xs uppercase tracking-widest" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">导入网易云歌单</div>
        <div class="mt-3 grid gap-3">
          <input
            class="meow-input"
            :class="state.isNight.value ? 'meow-input-night' : ''"
            type="text"
            placeholder="粘贴网易云歌单链接或 ID"
            v-model="state.neteaseImport.value.url"
          />
          <div class="playlist-select">
            <button class="playlist-select-btn" type="button" @click="toggleImport">
              <span class="truncate">{{ selectedImportLabel }}</span>
              <span class="playlist-select-arrow" :class="importOpen ? 'playlist-select-arrow-open' : ''">▾</span>
            </button>
            <transition name="select-pop">
              <div v-if="importOpen" class="playlist-select-menu">
                <button class="playlist-select-item" type="button" @click="pickImport('')">导入为新歌单</button>
                <button
                  v-for="plist in state.playlists.value"
                  :key="plist.id"
                  class="playlist-select-item"
                  type="button"
                  @click="pickImport(plist.id)"
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
            @click="state.importNeteasePlaylist"
            :disabled="state.neteaseImport.value.loading"
          >
            {{ state.neteaseImport.value.loading ? "导入中" : "开始导入" }}
          </button>
          <div v-if="state.neteaseImport.value.progress" class="text-xs" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
            {{ state.neteaseImport.value.progress }}
          </div>
          <div v-if="state.neteaseImport.value.error" class="text-xs text-[color:#e4547a]">{{ state.neteaseImport.value.error }}</div>
          <div v-if="state.neteaseImport.value.success" class="text-xs text-[color:#3aa889]">{{ state.neteaseImport.value.success }}</div>
        </div>
      </div>
    </div>

    <div class="mt-6 grid gap-4 md:grid-cols-3">
      <div v-if="state.dataLoading.value && !state.playlists.value.length" class="meow-card motion-card p-5 skeleton-card" :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''"></div>
      <div v-if="state.dataLoading.value && !state.playlists.value.length" class="meow-card motion-card p-5 skeleton-card" :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''"></div>
      <div v-if="state.dataLoading.value && !state.playlists.value.length" class="meow-card motion-card p-5 skeleton-card" :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''"></div>
      <article
        v-for="(playlist, idx) in state.playlists.value"
        :key="playlist.id || playlist.name"
        class="meow-card motion-card p-5 stagger-card"
        :style="{ '--float-delay': `${0.1 + idx * 0.25}s`, '--stagger': `${0.08 * (idx + 1)}s` }"
        :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''"
      >
        <div class="flex items-center justify-between">
          <span class="meow-pill">{{ playlist.mood }}</span>
          <span class="text-xs" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">{{ playlist.count }} 首</span>
        </div>
        <h3 class="mt-3 text-base font-600">{{ playlist.name }}</h3>
        <p class="mt-3 text-sm leading-relaxed" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">{{ playlist.desc }}</p>
        <div class="mt-4 flex flex-wrap items-center gap-2">
          <button class="meow-pill motion-press" type="button" @click="state.queueReplaceWithPlaylist(playlist.id)">载入队列</button>
          <router-link class="meow-pill motion-press" :class="state.isNight.value ? 'border-meow-night-line bg-meow-night-bg text-meow-night-ink' : ''" :to="`/playlists/${playlist.id}`">查看详情</router-link>
          <button class="meow-pill motion-press" type="button" @click="state.deletePlaylist(playlist.id)">删除</button>
        </div>
      </article>
    </div>
  </section>
</template>
