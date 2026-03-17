<script setup>
import { onMounted, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useMusicState } from "../composables/useMusicState";
defineOptions({ name: "PlaylistDetailPage" });

const route = useRoute();
const router = useRouter();
const state = useMusicState();

const loadDetail = async (id) => {
  if (!id) return;
  await state.fetchPlaylistDetail(id);
};

const onDeletePlaylist = async () => {
  if (!state.playlistDetail.value?.id) return;
  await state.deletePlaylist(state.playlistDetail.value.id);
  router.push("/playlists");
};

watch(
  () => route.params.id,
  (id) => {
    loadDetail(id);
  },
  { immediate: true }
);

onMounted(() => {
  if (route.params.id) {
    loadDetail(route.params.id);
  }
});
</script>

<template>
  <section id="playlist-detail" class="mt-12">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div>
        <div class="text-xs uppercase tracking-widest" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">歌单详情</div>
        <h2 class="mt-2 font-display text-2xl">{{ state.playlistDetail.value?.name || "歌单" }}</h2>
        <p class="mt-2 text-sm" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
          {{ state.playlistDetail.value?.desc || "没有描述" }}
        </p>
      </div>
      <div class="flex flex-wrap items-center gap-2">
        <span
          v-if="state.playlistDetailDurationPending.value && !state.playlistDetailLoading.value"
          class="meow-pill meow-pill-mini"
          :class="state.isNight.value ? 'border-meow-night-line bg-meow-night-bg text-meow-night-ink' : ''"
        >
          时长补齐中
        </span>
        <button class="meow-pill motion-press" type="button" @click="state.queueReplaceWithPlaylist(state.playlistDetail.value?.id)">载入队列</button>
        <button class="meow-pill motion-press" type="button" @click="onDeletePlaylist">删除歌单</button>
        <router-link class="meow-pill motion-press" :class="state.isNight.value ? 'border-meow-night-line bg-meow-night-bg text-meow-night-ink' : ''" to="/playlists">返回列表</router-link>
      </div>
    </div>

    <div
      class="meow-card motion-card playlist-detail-card mt-6 p-5"
      :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''"
    >
      <div class="library-head text-[11px] uppercase tracking-widest" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
        <span>曲目</span>
        <span>歌手</span>
        <span>专辑</span>
        <span>时长</span>
        <span>操作</span>
      </div>
      <div class="mt-4 space-y-2 playlist-detail-scroll">
      <div v-if="state.playlistDetailLoading.value" class="library-row skeleton-row" :class="state.isNight.value ? 'library-row-night' : 'library-row-day'"></div>
      <div v-if="state.playlistDetailLoading.value" class="library-row skeleton-row" :class="state.isNight.value ? 'library-row-night' : 'library-row-day'"></div>
        <div
          v-for="(track, idx) in state.playlistDetail.value?.tracks || []"
          :key="track.id"
          class="library-row stagger-card"
          :style="{ '--stagger': `${0.04 * (idx + 1)}s` }"
          :class="state.isNight.value ? 'library-row-night' : 'library-row-day'"
        >
          <span class="font-600">{{ track.title }}</span>
          <span class="text-sm" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">{{ track.artist }}</span>
          <span class="text-sm" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">{{ track.album }}</span>
          <span class="text-sm">{{ track.time }}</span>
          <div class="flex flex-wrap items-center gap-2">
            <button class="meow-pill motion-press" type="button" @click="state.queueAddItems([{ track_id: track.id, source: track.source }])">加入队列</button>
            <button class="meow-pill motion-press" type="button" @click="state.removeTrackFromPlaylist(state.playlistDetail.value?.id, track.id)">移出歌单</button>
          </div>
        </div>
      </div>
      <div v-if="!state.playlistDetailLoading.value && !(state.playlistDetail.value?.tracks || []).length" class="mt-4 text-sm" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
        该歌单暂无曲目
      </div>
      <div v-if="state.playlistDetailError.value" class="mt-4 text-xs text-[color:#e4547a]">{{ state.playlistDetailError.value }}</div>
    </div>
  </section>
</template>
