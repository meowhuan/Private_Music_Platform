<script setup>
import { useMusicState } from "../composables/useMusicState";
defineOptions({ name: "SettingsPage" });

const state = useMusicState();
</script>

<template>
  <section id="settings" class="mt-12 grid gap-6 md:grid-cols-2">
    <div class="meow-card motion-card p-5 stagger-card" :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''" :style="{ '--stagger': '0.08s' }">
      <div class="flex items-center justify-between gap-3">
        <div>
          <div class="text-[11px] uppercase tracking-widest" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">后端连接</div>
          <div class="mt-1 text-sm" :class="state.isNight.value ? 'text-meow-night-ink' : 'text-meow-ink'">
            {{ state.apiBase || "未设置 VITE_API_BASE" }}
          </div>
        </div>
        <button
          v-if="state.isAuthed.value"
          class="meow-pill motion-press px-2 py-0.5 text-[11px]"
          :class="state.isNight.value ? 'border-meow-night-line bg-meow-night-bg text-meow-night-ink' : ''"
          type="button"
          @click="state.fetchAll(true)"
          :disabled="state.dataLoading.value"
        >
          {{ state.dataLoading.value ? "同步中" : "刷新数据" }}
        </button>
      </div>

      <div v-if="!state.apiReady" class="mt-3 text-xs text-[color:#e4547a]">
        请在 `.env` 中设置 `VITE_API_BASE`（例如 `http://127.0.0.1:8080`）。
      </div>
      <div v-else-if="!state.apiBaseValid" class="mt-3 text-xs text-[color:#e4547a]">
        `VITE_API_BASE` 必须包含 `http://` 或 `https://`。
      </div>
      <div v-else-if="!state.isAuthed.value" class="mt-3 grid gap-2 sm:grid-cols-[1fr_1fr_auto]">
        <input
          v-model="state.loginForm.value.username"
          class="meow-input"
          :class="state.isNight.value ? 'meow-input-night' : ''"
          type="text"
          placeholder="用户名"
        />
        <input
          v-model="state.loginForm.value.password"
          class="meow-input"
          :class="state.isNight.value ? 'meow-input-night' : ''"
          type="password"
          placeholder="密码"
        />
        <button
          class="meow-btn-primary motion-press"
          :class="state.isNight.value ? 'bg-meow-night-accent text-meow-night-bg' : ''"
          type="button"
          @click="state.login"
          :disabled="state.authLoading.value"
        >
          {{ state.authLoading.value ? "登录中" : "登录" }}
        </button>
      </div>
      <div v-else class="mt-3 flex flex-wrap items-center gap-3 text-xs" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
        <span class="meow-pill">已登录</span>
        <span class="meow-pill meow-pill-mini">用户：{{ state.authUsername.value || state.loginForm.value.username || "admin" }}</span>
        <span v-if="state.isAdmin.value" class="meow-pill meow-pill-mini meow-pill-admin">管理员</span>
        <button class="meow-pill motion-press" type="button" @click="state.logout">退出</button>
      </div>
      <div v-if="state.authError.value" class="mt-2 text-xs text-[color:#e4547a]">{{ state.authError.value }}</div>
      <div v-if="state.dataError.value" class="mt-2 text-xs text-[color:#e4547a]">{{ state.dataError.value }}</div>
    </div>

    <div class="meow-card motion-card p-5 stagger-card" :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''" :style="{ '--stagger': '0.22s' }">
      <div class="text-[11px] uppercase tracking-widest" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">管理员注册用户</div>
      <div class="mt-2 text-xs" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
        仅管理员可操作注册，普通用户请联系站长。
      </div>
      <div class="mt-1 text-xs" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">邮箱：meowhuan@meowra.cn</div>
      <div class="mt-3 grid gap-2 sm:grid-cols-[1fr_1fr_auto]">
        <input
          v-model="state.userCreateForm.value.username"
          class="meow-input"
          :class="state.isNight.value ? 'meow-input-night' : ''"
          type="text"
          placeholder="新用户名"
        />
        <input
          v-model="state.userCreateForm.value.password"
          class="meow-input"
          :class="state.isNight.value ? 'meow-input-night' : ''"
          type="password"
          placeholder="初始密码"
        />
        <button
          class="meow-btn-primary motion-press"
          :class="state.isNight.value ? 'bg-meow-night-accent text-meow-night-bg' : ''"
          type="button"
          @click="state.createUser"
          :disabled="state.userCreateLoading.value"
        >
          {{ state.userCreateLoading.value ? "创建中" : "创建用户" }}
        </button>
      </div>
      <div v-if="state.userCreateError.value" class="mt-2 text-xs text-[color:#e4547a]">{{ state.userCreateError.value }}</div>
      <div v-if="state.userCreateSuccess.value" class="mt-2 text-xs text-[color:#3aa889]">{{ state.userCreateSuccess.value }}</div>

      <div v-if="state.isAdmin.value" class="mt-5 grid gap-2">
        <div class="text-[11px] uppercase tracking-widest" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">管理员重置密码</div>
        <div class="grid gap-2 sm:grid-cols-[1fr_1fr_auto]">
          <input
            v-model="state.adminResetForm.value.username"
            class="meow-input"
            :class="state.isNight.value ? 'meow-input-night' : ''"
            type="text"
            placeholder="用户名"
          />
          <input
            v-model="state.adminResetForm.value.next"
            class="meow-input"
            :class="state.isNight.value ? 'meow-input-night' : ''"
            type="password"
            placeholder="新密码"
          />
          <button
            class="meow-btn-primary motion-press"
            :class="state.isNight.value ? 'bg-meow-night-accent text-meow-night-bg' : ''"
            type="button"
            @click="state.adminResetPassword"
            :disabled="state.adminResetLoading.value"
          >
            {{ state.adminResetLoading.value ? "重置中" : "重置密码" }}
          </button>
        </div>
        <div v-if="state.adminResetError.value" class="text-xs text-[color:#e4547a]">{{ state.adminResetError.value }}</div>
        <div v-if="state.adminResetSuccess.value" class="text-xs text-[color:#3aa889]">{{ state.adminResetSuccess.value }}</div>
      </div>
    </div>

    <div class="meow-card motion-card p-5 stagger-card" :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''" :style="{ '--stagger': '0.28s' }">
      <div class="text-[11px] uppercase tracking-widest" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">修改密码</div>
      <div v-if="state.isAdmin.value" class="mt-2 text-xs" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
        管理员密码需通过环境变量修改。
      </div>
      <div v-else class="mt-3 grid gap-2 sm:grid-cols-[1fr_1fr_auto]">
        <input
          v-model="state.passwordForm.value.current"
          class="meow-input"
          :class="state.isNight.value ? 'meow-input-night' : ''"
          type="password"
          placeholder="当前密码"
        />
        <input
          v-model="state.passwordForm.value.next"
          class="meow-input"
          :class="state.isNight.value ? 'meow-input-night' : ''"
          type="password"
          placeholder="新密码"
        />
        <button
          class="meow-btn-primary motion-press"
          :class="state.isNight.value ? 'bg-meow-night-accent text-meow-night-bg' : ''"
          type="button"
          @click="state.changePassword"
          :disabled="state.passwordLoading.value"
        >
          {{ state.passwordLoading.value ? "修改中" : "修改密码" }}
        </button>
      </div>
      <div v-if="state.passwordError.value" class="mt-2 text-xs text-[color:#e4547a]">{{ state.passwordError.value }}</div>
      <div v-if="state.passwordSuccess.value" class="mt-2 text-xs text-[color:#3aa889]">{{ state.passwordSuccess.value }}</div>
    </div>

    <div class="meow-card motion-card p-5 stagger-card" :class="state.isNight.value ? 'bg-meow-night-card/80 border-meow-night-line' : ''" :style="{ '--stagger': '0.16s' }">
      <div class="flex items-center justify-between gap-3">
        <div>
          <div class="text-[11px] uppercase tracking-widest" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">网易云账号</div>
          <div class="mt-1 text-sm" :class="state.isNight.value ? 'text-meow-night-ink' : 'text-meow-ink'">
            {{ state.neteaseProfile.value?.nickname || (state.neteaseCookie.value ? "已登录" : "未登录") }}
          </div>
        </div>
        <button
          v-if="state.neteaseCookie.value"
          class="meow-pill motion-press"
          type="button"
          @click="state.neteaseLogout"
        >
          退出
        </button>
      </div>

      <div v-if="!state.neteaseCookie.value" class="mt-4 grid gap-3">
        <div class="flex flex-wrap items-center gap-2 text-xs" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
          <span>当前使用：</span>
          <span class="meow-pill meow-pill-mini">
            {{ state.neteaseCookieMode.value === "user" ? "个人网易云账户（隔离）" : "管理员网易云账户（全局）" }}
          </span>
          <span v-if="state.isAdmin.value" class="meow-pill meow-pill-mini meow-pill-admin">管理员</span>
          <button
            v-if="!state.isAdmin.value"
            class="meow-pill meow-pill-mini motion-press"
            type="button"
            @click="state.setNeteaseCookieMode(state.neteaseCookieMode.value === 'user' ? 'admin' : 'user')"
          >
            切换
          </button>
        </div>
        <div class="text-xs" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
          轮询二维码状态：800 过期、801 等待扫码、802 待确认、803 授权成功（803 返回 cookie）。<br />
          若扫码后返回 502，将自动追加 `noCookie=true` 再重试。
        </div>
        <div class="qr-box">
          <div class="text-[11px] uppercase tracking-widest" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">扫码登录</div>
          <button class="meow-pill motion-press mt-2" type="button" @click="state.startQrLogin">生成二维码</button>
          <div v-if="state.qr.value.qrimg" class="mt-3 qr-image">
            <img :src="state.qr.value.qrimg" alt="QR" />
          </div>
          <div class="mt-2 text-xs" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
            {{ state.qr.value.message }}
          </div>
          <button class="meow-pill motion-press mt-2" type="button" @click="state.getLoginStatus(state.neteaseCookie.value)">刷新登录状态</button>
          <div v-if="state.qrLast.value.at" class="mt-2 text-[11px]" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
            最后响应：{{ state.qrLast.value.at }} · code={{ state.qrLast.value.code }} · noCookie={{ state.qrLast.value.noCookie }}
          </div>
          <pre v-if="state.qrLast.value.raw" class="mt-2 text-[11px] qr-debug">{{ JSON.stringify(state.qrLast.value.raw, null, 2) }}</pre>
          <div v-if="state.neteaseLoginError.value" class="mt-2 text-xs text-[color:#e4547a]">{{ state.neteaseLoginError.value }}</div>
        </div>
      </div>
      <div v-else class="mt-3 flex flex-wrap items-center gap-2 text-xs" :class="state.isNight.value ? 'text-meow-night-soft' : 'text-meow-soft'">
        <span>当前使用：</span>
        <span class="meow-pill meow-pill-mini">
          {{ state.neteaseCookieMode.value === "user" ? "个人网易云账户（隔离）" : "管理员网易云账户（全局）" }}
        </span>
        <span v-if="state.isAdmin.value" class="meow-pill meow-pill-mini meow-pill-admin">管理员</span>
        <button
          v-if="!state.isAdmin.value"
          class="meow-pill meow-pill-mini motion-press"
          type="button"
          @click="state.setNeteaseCookieMode(state.neteaseCookieMode.value === 'user' ? 'admin' : 'user')"
        >
          切换
        </button>
      </div>
    </div>
  </section>
</template>
