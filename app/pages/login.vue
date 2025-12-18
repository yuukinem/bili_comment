<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { NCard, NButton, NSpin, NResult, useMessage } from 'naive-ui'
import { useAuthStore } from '~/stores/auth'

const router = useRouter()
const message = useMessage()
const authStore = useAuthStore()

const pollTimer = ref<ReturnType<typeof setInterval> | null>(null)

// 获取二维码
async function getQrCode() {
  try {
    await authStore.getLoginQrCode()
    startPolling()
  } catch (error) {
    message.error(String(error))
  }
}

// 开始轮询
function startPolling() {
  stopPolling()
  pollTimer.value = setInterval(async () => {
    try {
      const result = await authStore.pollLoginStatus()
      if (result?.status === 'confirmed') {
        stopPolling()
        message.success('登录成功')
        router.push('/')
      } else if (result?.status === 'expired') {
        stopPolling()
        message.warning('二维码已过期，请刷新')
      }
    } catch {
      // 忽略轮询错误
    }
  }, 2000)
}

// 停止轮询
function stopPolling() {
  if (pollTimer.value) {
    clearInterval(pollTimer.value)
    pollTimer.value = null
  }
}

// 刷新二维码
async function refreshQrCode() {
  stopPolling()
  await getQrCode()
}

onMounted(async () => {
  // 检查是否已登录
  const isLoggedIn = await authStore.checkLogin()
  if (isLoggedIn) {
    await authStore.fetchUserInfo()
    router.push('/')
    return
  }

  // 获取二维码
  await getQrCode()
})

onUnmounted(() => {
  stopPolling()
})
</script>

<template>
  <div class="login-page">
    <NCard class="login-card" title="B站登录" :bordered="false">
      <div class="qrcode-container">
        <!-- 加载中 -->
        <div v-if="authStore.isLoading" class="loading">
          <NSpin size="large" />
          <p>正在加载...</p>
        </div>

        <!-- 二维码 -->
        <template v-else-if="authStore.qrCode">
          <div
            v-if="authStore.loginStatus === 'expired'"
            class="qrcode-expired"
          >
            <NResult status="warning" title="二维码已过期">
              <template #footer>
                <NButton type="primary" @click="refreshQrCode">
                  刷新二维码
                </NButton>
              </template>
            </NResult>
          </div>

          <div v-else class="qrcode-wrapper">
            <img
              :src="authStore.qrCode.image_base64"
              alt="登录二维码"
              class="qrcode-image"
            />
            <p class="status-text">{{ authStore.loginMessage }}</p>

            <div v-if="authStore.loginStatus === 'scanned'" class="scanned-tip">
              <NSpin size="small" />
              <span>请在手机上确认登录</span>
            </div>
          </div>
        </template>

        <!-- 错误状态 -->
        <NResult
          v-else-if="authStore.loginStatus === 'error'"
          status="error"
          :title="authStore.loginMessage"
        >
          <template #footer>
            <NButton type="primary" @click="getQrCode">重试</NButton>
          </template>
        </NResult>
      </div>

      <div class="tips">
        <p>请使用哔哩哔哩APP扫描二维码登录</p>
        <p class="sub-tip">登录后可使用搜索视频和发送评论功能</p>
      </div>
    </NCard>
  </div>
</template>

<style scoped>
.login-page {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100vh;
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
}

.login-card {
  width: 400px;
  text-align: center;
}

.qrcode-container {
  min-height: 280px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.qrcode-wrapper {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.qrcode-image {
  width: 200px;
  height: 200px;
  border-radius: 8px;
  background: white;
  padding: 8px;
}

.status-text {
  margin-top: 16px;
  color: #999;
}

.scanned-tip {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 12px;
  color: #18a058;
}

.tips {
  margin-top: 24px;
  color: #666;
  font-size: 14px;
}

.sub-tip {
  font-size: 12px;
  color: #888;
  margin-top: 8px;
}

.qrcode-expired {
  padding: 20px;
}
</style>
