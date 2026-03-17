<script setup>
import { ref } from "vue";
import { useMusicState } from "../composables/useMusicState";
defineOptions({ name: "DevicesPage" });

const state = useMusicState();
const draggingIndex = ref(null);

const onDragStart = (index) => {
  draggingIndex.value = index;
};

const onDragOver = (event) => {
  event.preventDefault();
};

const onDrop = async (index) => {
  if (draggingIndex.value === null || draggingIndex.value === index) return;
  const next = [...state.queue.value];
  const [moved] = next.splice(draggingIndex.value, 1);
  next.splice(index, 0, moved);
  state.queue.value = next;
  draggingIndex.value = null;
  const order = next.map((item) => item.queue_id).filter(Boolean);
  if (order.length) {
    await state.queueReorder(order);
  }
};

const onDragEnd = () => {
  draggingIndex.value = null;
};
</script>

<template>
  <section id="devices" class="mt-12 grid gap-6 md:grid-cols-[1.1fr_0.9fr]">
    <div>
      <h2 class="font-display text-2xl">设备与同步</h2>
      <div class="mt-6 space-y-3">
        <div class="meow-card motion-card p-5" :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''">
          <div class="text-xs uppercase tracking-widest" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">添加设备</div>
          <p class="mt-2 text-xs" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
            仅管理员可添加设备，使用者需联系我获取授权。
          </p>
          <div class="mt-3 grid gap-3">
            <input
              class="meow-input"
              :class="state.isNight.value ? 'meow-input-night' : ''"
              type="text"
              placeholder="设备名称"
              v-model="state.deviceCreateForm.value.name"
            />
            <input
              class="meow-input"
              :class="state.isNight.value ? 'meow-input-night' : ''"
              type="text"
              placeholder="设备说明（可选）"
              v-model="state.deviceCreateForm.value.desc"
            />
            <button
              class="meow-btn-primary motion-press"
              :class="state.isNight.value ? 'bg-meow-night-accent text-meow-night-bg' : ''"
              type="button"
              @click="state.createDevice"
              :disabled="state.deviceCreateLoading.value"
            >
              {{ state.deviceCreateLoading.value ? "创建中" : "创建设备" }}
            </button>
            <div v-if="state.deviceCreateError.value" class="text-xs text-[color:#e4547a]">{{ state.deviceCreateError.value }}</div>
            <div v-if="state.deviceCreateResult.value" class="text-xs" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
              设备已创建，密钥仅展示一次：
              <span class="meow-pill meow-pill-mini ml-1">{{ state.deviceCreateResult.value.token }}</span>
            </div>
          </div>
        </div>
        <div v-if="state.dataLoading.value && !state.devices.value.length" class="meow-card motion-card p-5 skeleton-card" :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''"></div>
        <div v-if="state.dataLoading.value && !state.devices.value.length" class="meow-card motion-card p-5 skeleton-card" :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''"></div>
        <div
          v-for="(device, idx) in state.devices.value"
          :key="device.id || device.name"
          class="meow-card motion-card device-card p-5 stagger-card"
          :style="{ '--stagger': `${0.06 * (idx + 1)}s` }"
          :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''"
        >
          <div class="flex items-start justify-between gap-3">
            <div>
              <div class="text-base font-600">{{ device.name }}</div>
              <div class="mt-2 text-sm device-desc" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">{{ device.desc }}</div>
              <div class="device-meta">
                <span class="device-chip">在线 {{ state.formatRelativeTime(device.last_seen_at) }}</span>
                <span class="device-chip">同步 {{ device.last_sync_at ? state.formatRelativeTime(device.last_sync_at) : "未同步" }}</span>
                <span class="device-chip">缓存 {{ device.cache_size_mb ?? 0 }}MB</span>
                <span class="device-chip">曲目 {{ device.synced_tracks ?? 0 }} 首</span>
              </div>
              <div v-if="device.playing_title" class="device-playing" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
                播放中：{{ device.playing_title }} {{ device.playing_artist ? `· ${device.playing_artist}` : "" }}
                <span v-if="device.playback_progress !== null && device.playback_progress !== undefined">
                  · {{ device.playback_progress }}%
                </span>
              </div>
            </div>
            <div class="flex flex-col items-end gap-2">
              <span class="meow-pill">{{ device.status }}</span>
              <button class="meow-pill motion-press" type="button" @click="state.syncDevice(device.id)">立即同步</button>
              <button class="meow-pill motion-press" type="button" @click="state.deleteDevice(device.id)">解除绑定</button>
            </div>
          </div>
        </div>
      </div>
    </div>
    <div
      class="meow-card motion-card device-queue-card p-6"
      :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''"
    >
      <div class="flex items-center justify-between gap-2">
        <h3 class="text-lg font-600">播放队列</h3>
        <button class="meow-pill motion-press" type="button" @click="state.queueClearAll">清空队列</button>
      </div>
      <div class="mt-4 space-y-3 device-queue-list">
        <div v-if="state.dataLoading.value && !state.queue.value.length" class="queue-row skeleton-row"></div>
        <div v-if="state.dataLoading.value && !state.queue.value.length" class="queue-row skeleton-row"></div>
        <div
          v-for="(item, idx) in state.queue.value"
          :key="item.queue_id || item.id || item.title"
          class="queue-row stagger-card"
          :style="{ '--stagger': `${0.05 * (idx + 1)}s` }"
          :class="[
            state.currentIndex.value === idx ? 'queue-row-active' : '',
            draggingIndex === idx ? 'queue-row-dragging' : ''
          ]"
          draggable="true"
          @dragstart="onDragStart(idx)"
          @dragover="onDragOver"
          @drop="onDrop(idx)"
          @dragend="onDragEnd"
          @click="state.playQueueIndex(idx)"
        >
          <div class="queue-title">{{ item.title }}</div>
          <div class="text-xs" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">{{ item.artist }}</div>
          <div class="queue-time">{{ item.time }}</div>
        </div>
      </div>
      <div v-if="!state.queue.value.length && !state.dataLoading.value" class="mt-4 text-xs" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
        队列为空
      </div>
      <div
        v-if="state.nextUpText.value"
        class="mt-4 text-xs"
        :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'"
      >
        {{ state.nextUpText.value }}
      </div>
    </div>
  </section>
</template>
