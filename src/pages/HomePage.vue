<script setup>
import { computed, nextTick, onMounted, ref, watch } from "vue";
import { useMusicState } from "../composables/useMusicState";
defineOptions({ name: "HomePage" });

const state = useMusicState();
const displayTrack = computed(() => state.currentTrack.value || state.nowPlaying.value);
const lyricPanelEl = ref(null);
const coverUrl = computed(() => state.resolveCoverUrl(displayTrack.value?.cover_url || ""));
const greeting = computed(() => {
  const hour = new Date().getHours();
  if (hour >= 5 && hour < 11) return "早上好";
  if (hour >= 11 && hour < 17) return "中午好";
  if (hour >= 17 && hour < 22) return "晚上好";
  return "深夜好";
});
const timeDigits = computed(() => {
  const raw = state.time.value || "00:00:00";
  const [h, m, s] = raw.split(":");
  const safe = (v) => (v || "00").padStart(2, "0");
  const hh = safe(h);
  const mm = safe(m);
  const ss = safe(s);
  return {
    h1: hh[0],
    h2: hh[1],
    m1: mm[0],
    m2: mm[1],
    s1: ss[0],
    s2: ss[1],
    tick: raw
  };
});
const prevDigits = ref({ ...timeDigits.value });
const flipFlags = ref({
  h1: false,
  h2: false,
  m1: false,
  m2: false,
  s1: false,
  s2: false
});

watch(
  () => timeDigits.value,
  (next, prev) => {
    const last = prev || prevDigits.value;
    prevDigits.value = { ...last };
    const keys = ["h1", "h2", "m1", "m2", "s1", "s2"];
    let changed = false;
    keys.forEach((key) => {
      const flip = last[key] !== next[key];
      flipFlags.value[key] = flip;
      if (flip) changed = true;
    });
    if (changed) {
      window.setTimeout(() => {
        keys.forEach((key) => {
          flipFlags.value[key] = false;
        });
        prevDigits.value = { ...next };
      }, 360);
    } else {
      prevDigits.value = { ...next };
    }
  },
  { immediate: true }
);
const trendSeries = computed(() => {
  const values = state.playbackTrend.value?.values;
  if (values?.length) return values;
  return [12, 18, 10, 22, 14, 26, 16, 24, 19];
});
const trendPath = computed(() => {
  const series = trendSeries.value;
  const width = 148;
  const height = 48;
  const pad = 4;
  const max = Math.max(...series);
  const min = Math.min(...series);
  const scale = (value) => {
    if (max === min) return height / 2;
    return height - ((value - min) / (max - min)) * height;
  };
  return series
    .map((value, index) => {
      const x = pad + (index / (series.length - 1)) * (width - pad * 2);
      const y = pad + scale(value);
      return `${index === 0 ? "M" : "L"} ${x} ${y}`;
    })
    .join(" ");
});

const scrollLyricToActive = async () => {
  await nextTick();
  const wrap = lyricPanelEl.value;
  if (!wrap) return;
  const active = wrap.querySelector(".lyric-line.active");
  if (!active) return;
  if (state.currentSeconds.value < 1.5 && state.currentLyricIndex.value <= 1) return;
  const maxTop = wrap.scrollHeight - wrap.clientHeight;
  const anchor = wrap.clientHeight * 0.4;
  const top = Math.max(0, Math.min(maxTop, active.offsetTop - anchor));
  const delta = Math.abs(wrap.scrollTop - top);
  if (delta < 24) return;
  wrap.scrollTo({ top, behavior: "smooth" });
};

watch(
  () => state.currentLyricIndex.value,
  () => {
    scrollLyricToActive();
  }
);

watch(
  () => state.currentLyric.value,
  () => {
    scrollLyricToActive();
  }
);

onMounted(() => {
  scrollLyricToActive();
});

const onProgressClick = (event) => {
  const rect = event.currentTarget.getBoundingClientRect();
  const percent = (event.clientX - rect.left) / rect.width;
  state.seekTo(percent);
};
</script>

<template>
  <div class="page-stack">
    <section id="home" class="home-grid mt-12 grid gap-8 md:grid-cols-[1.15fr_0.85fr]">
    <div class="home-stack">
      <span class="meow-pill meow-pill-strong">🎧 Private Music Hub</span>
      <h1 class="font-display text-4xl leading-tight sm:text-5xl">
        只属于你的音乐空间。
      </h1>
      <p class="text-base leading-relaxed" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
        这里是只给你留灯的音乐角落，把心情、夜晚与收藏慢慢收进来，让每一次播放都像一场小小的仪式。
      </p>
      <div class="flex flex-wrap gap-3">
        <button class="meow-btn-primary motion-press" :class="state.isNight.value ? 'bg-meow-night-accent text-meow-night-bg' : ''" @click="state.startPlayback">开始播放</button>
        <router-link to="/library" class="meow-btn-ghost motion-press" :class="state.isNight.value ? 'border-meow-night-line text-meow-night-ink hover:bg-meow-night-card/80' : ''">导入曲库</router-link>
        <router-link to="/devices" class="meow-btn-ghost motion-press" :class="state.isNight.value ? 'border-meow-night-line text-meow-night-ink hover:bg-meow-night-card/80' : ''">同步设备</router-link>
      </div>
      <div class="flex flex-wrap gap-4 text-xs" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
        <div class="flex items-center gap-2">
          <span class="status-dot status-dot-ok"></span>
          <span>云端同步正常</span>
        </div>
        <div>更新时间 {{ state.time.value }} · {{ state.date.value }}</div>
      </div>

      <div
        class="meow-card motion-card lyric-panel"
        :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''"
      >
        <div class="lyric-bg" :style="coverUrl ? { backgroundImage: `url(${coverUrl})` } : {}"></div>
        <div class="lyric-overlay" :class="state.isNight.value ? 'lyric-overlay-night' : 'lyric-overlay-day'"></div>
        <div class="lyric-fade lyric-fade-top"></div>
        <div class="lyric-fade lyric-fade-bottom"></div>

        <div class="lyric-header">
          <div class="text-[11px] uppercase tracking-widest" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">歌词</div>
          <span class="meow-pill">{{ state.currentTrack.value ? "正在滚动" : "未播放" }}</span>
        </div>
        <div class="lyric-spectrum-bg" :class="state.isNight.value ? 'lyric-spectrum-night' : ''">
          <div class="lyric-spectrum-bars">
            <span
              v-for="(bar, idx) in state.spectrum.value"
              :key="`s-${idx}`"
              class="lyric-spectrum-bar"
              :style="{ height: `${10 + bar * 42}px` }"
            ></span>
          </div>
        </div>

        <div v-if="state.currentLyricLoading.value" class="lyric-status text-xs" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
          歌词加载中...
        </div>
        <div v-else-if="state.currentLyricError.value" class="lyric-status text-xs text-[color:#e4547a]">{{ state.currentLyricError.value }}</div>
        <div
          v-else
          ref="lyricPanelEl"
          class="lyric-scroll"
        >
          <div
            v-for="(line, idx) in state.lyricLines.value"
            :key="`${line.time}-${idx}`"
            class="lyric-line"
            :class="state.currentLyricIndex.value === idx ? 'active' : ''"
          >
            <div class="lyric-original">{{ line.text }}</div>
            <div v-if="line.trans" class="lyric-translation">{{ line.trans }}</div>
          </div>
          <div v-if="!state.lyricLines.value.length" class="text-xs" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
            暂无歌词
          </div>
        </div>
      </div>
    </div>

    <div class="player-stack">
      <div class="player-group">
      <div class="player-greeting-row">
        <div class="player-greeting" :class="state.isNight.value ? 'player-greeting-night' : ''">
          <div class="player-greeting-label" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">现在是</div>
          <div class="flip-clock">
            <div class="flip-card" :class="{ 'is-flipping': flipFlags.h1 }">
              <span class="flip-face flip-face-top">{{ timeDigits.h1 }}</span>
              <span class="flip-face flip-face-bottom">{{ timeDigits.h1 }}</span>
              <span class="flip-anim flip-anim-top">{{ prevDigits.h1 }}</span>
            </div>
            <div class="flip-card" :class="{ 'is-flipping': flipFlags.h2 }">
              <span class="flip-face flip-face-top">{{ timeDigits.h2 }}</span>
              <span class="flip-face flip-face-bottom">{{ timeDigits.h2 }}</span>
              <span class="flip-anim flip-anim-top">{{ prevDigits.h2 }}</span>
            </div>
            <span class="flip-sep">:</span>
            <div class="flip-card" :class="{ 'is-flipping': flipFlags.m1 }">
              <span class="flip-face flip-face-top">{{ timeDigits.m1 }}</span>
              <span class="flip-face flip-face-bottom">{{ timeDigits.m1 }}</span>
              <span class="flip-anim flip-anim-top">{{ prevDigits.m1 }}</span>
            </div>
            <div class="flip-card" :class="{ 'is-flipping': flipFlags.m2 }">
              <span class="flip-face flip-face-top">{{ timeDigits.m2 }}</span>
              <span class="flip-face flip-face-bottom">{{ timeDigits.m2 }}</span>
              <span class="flip-anim flip-anim-top">{{ prevDigits.m2 }}</span>
            </div>
            <span class="flip-sep">:</span>
            <div class="flip-card" :class="{ 'is-flipping': flipFlags.s1 }">
              <span class="flip-face flip-face-top">{{ timeDigits.s1 }}</span>
              <span class="flip-face flip-face-bottom">{{ timeDigits.s1 }}</span>
              <span class="flip-anim flip-anim-top">{{ prevDigits.s1 }}</span>
            </div>
            <div class="flip-card" :class="{ 'is-flipping': flipFlags.s2 }">
              <span class="flip-face flip-face-top">{{ timeDigits.s2 }}</span>
              <span class="flip-face flip-face-bottom">{{ timeDigits.s2 }}</span>
              <span class="flip-anim flip-anim-top">{{ prevDigits.s2 }}</span>
            </div>
          </div>
          <div class="player-greeting-sub" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">{{ state.date.value }} · {{ greeting }}</div>
        </div>
        <div class="player-trend" :class="state.isNight.value ? 'player-trend-night' : ''">
          <div class="text-[11px] uppercase tracking-widest" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">听歌趋势</div>
          <svg viewBox="0 0 148 56" class="player-trend-chart" aria-hidden="true">
            <path class="player-trend-line" :d="trendPath"></path>
          </svg>
          <div class="text-[11px]" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">{{ state.playbackTrend.value?.label || "最近 9 天" }}</div>
        </div>
      </div>
        <div
          class="meow-card motion-card player-card p-6 stagger-card"
          :style="{ '--float-delay': '0.2s', '--stagger': '0.12s' }"
          :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''"
        >
      <div class="flex items-start gap-4">
        <div class="player-cover" :class="state.isNight.value ? 'player-cover-night' : 'player-cover-day'">
          <img v-if="coverUrl" :src="coverUrl" alt="cover" />
        </div>
        <div class="flex-1">
          <div class="text-[11px] uppercase tracking-widest" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">正在播放</div>
          <div class="mt-2 text-lg font-600">{{ displayTrack.title }}</div>
          <div class="text-sm" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">{{ displayTrack.artist }} · {{ displayTrack.album }}</div>
          <div class="mt-3 text-[11px]" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">{{ state.qualityText.value }}</div>
        </div>
        <span class="meow-pill">{{ state.nowPlaying.value.mood || displayTrack.mood }}</span>
      </div>
      <div class="mt-4">
        <div class="music-progress" @click="onProgressClick">
          <div class="music-progress-fill" :style="{ width: state.timeProgress.value }"></div>
        </div>
        <div class="mt-2 flex items-center justify-between text-[11px]" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
          <span>{{ state.currentTimeText.value }}</span>
          <span>{{ state.durationText.value }}</span>
        </div>
      </div>
      <div class="mt-4 flex items-center justify-between">
        <div class="music-controls">
          <button class="control-btn" type="button" @click="state.playPrev(true)">⏮</button>
          <button class="control-btn control-play" type="button" @click="state.togglePlay">{{ state.isPlaying.value ? "⏸" : "▶" }}</button>
          <button class="control-btn" type="button" @click="state.playNext(true)">⏭</button>
        </div>
        <div class="music-eq">
          <span class="eq-bar"></span>
          <span class="eq-bar"></span>
          <span class="eq-bar"></span>
          <span class="eq-bar"></span>
        </div>
        <button class="control-btn control-queue" type="button" @click="state.queueDockOpen.value = true">≡</button>
      </div>
      <div class="mt-4 flex flex-wrap items-center justify-between gap-3 text-xs" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
        <label class="flex items-center gap-2">
          音量
          <input class="volume-slider" type="range" min="0" max="1" step="0.01" :value="state.volume.value" @input="state.setVolume($event.target.valueAsNumber)" />
        </label>
        <div class="flex items-center gap-2">
          <span>播放模式</span>
          <div class="playmode-group">
            <button
              class="playmode-btn"
              :class="state.playMode.value === 'order' ? 'playmode-btn-active' : ''"
              type="button"
              title="顺序"
              @click="state.setPlayMode('order')"
            >SEQ</button>
            <button
              class="playmode-btn"
              :class="state.playMode.value === 'repeat-all' ? 'playmode-btn-active' : ''"
              type="button"
              title="列表循环"
              @click="state.setPlayMode('repeat-all')"
            >RPT</button>
            <button
              class="playmode-btn"
              :class="state.playMode.value === 'repeat-one' ? 'playmode-btn-active' : ''"
              type="button"
              title="单曲循环"
              @click="state.setPlayMode('repeat-one')"
            >ONE</button>
            <button
              class="playmode-btn"
              :class="state.playMode.value === 'shuffle' ? 'playmode-btn-active' : ''"
              type="button"
              title="随机"
              @click="state.setPlayMode('shuffle')"
            >SHF</button>
          </div>
        </div>
      </div>
      <div v-if="state.playError.value" class="mt-2 text-xs text-[color:#e4547a]">{{ state.playError.value }}</div>
        </div>
      </div>
    </div>
    </section>

    <section id="stats" class="mt-16">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <h2 class="font-display text-2xl">本周统计</h2>
      <span class="meow-pill meow-pill-strong">{{ state.dataLoading.value ? "同步中" : "同步正常" }}</span>
    </div>
    <div class="mt-6 grid gap-4 md:grid-cols-4">
      <div v-if="state.dataLoading.value && !state.stats.value.length" class="meow-card motion-card p-5 skeleton-card" :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''"></div>
      <div v-if="state.dataLoading.value && !state.stats.value.length" class="meow-card motion-card p-5 skeleton-card" :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''"></div>
      <div v-if="state.dataLoading.value && !state.stats.value.length" class="meow-card motion-card p-5 skeleton-card" :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''"></div>
      <div v-if="state.dataLoading.value && !state.stats.value.length" class="meow-card motion-card p-5 skeleton-card" :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''"></div>
      <div
        v-for="(stat, idx) in state.stats.value"
        :key="stat.label"
        class="meow-card motion-card p-5 stagger-card"
        :style="{ '--stagger': `${0.08 * (idx + 1)}s` }"
        :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''"
      >
        <div class="text-[11px] uppercase tracking-widest" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">{{ stat.label }}</div>
        <div class="mt-3 text-2xl font-700">{{ stat.value }}</div>
      </div>
    </div>
    </section>
  </div>
</template>
