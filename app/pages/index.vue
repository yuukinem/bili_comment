<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import {
  NLayout,
  NLayoutHeader,
  NLayoutContent,
  NSpace,
  NInput,
  NButton,
  NSelect,
  NCheckbox,
  NCard,
  NImage,
  NTag,
  NPagination,
  NModal,
  NEmpty,
  NSpin,
  NAvatar,
  NDropdown,
  NBadge,
  useMessage,
  useDialog,
} from 'naive-ui'
import { useAuthStore } from '~/stores/auth'
import { useSearchStore } from '~/stores/search'
import { useCommentStore } from '~/stores/comment'
import { useTemplateStore } from '~/stores/template'
import { SEARCH_ORDER_OPTIONS } from '~/types/bilibili'
import type { VideoItem, CommentTemplate } from '~/types/bilibili'

const router = useRouter()
const message = useMessage()
const dialog = useDialog()

const authStore = useAuthStore()
const searchStore = useSearchStore()
const commentStore = useCommentStore()
const templateStore = useTemplateStore()

// 搜索关键词
const searchKeyword = ref('')

// 评论弹窗
const showCommentModal = ref(false)
const commentContent = ref('')
const selectedTemplate = ref<string | null>(null)
const commentTarget = ref<'single' | 'batch'>('batch')
const singleVideo = ref<VideoItem | null>(null)

// 排序选项
const orderOptions = SEARCH_ORDER_OPTIONS.map((o) => ({
  label: o.label,
  value: o.value,
}))

// 用户下拉菜单
const userMenuOptions = [
  { label: '模板管理', key: 'templates' },
  { label: '退出登录', key: 'logout' },
]

// 格式化播放量
function formatPlay(num: number): string {
  if (num >= 10000) {
    return (num / 10000).toFixed(1) + '万'
  }
  return num.toString()
}

// 格式化时间
function formatDate(timestamp: number): string {
  const date = new Date(timestamp * 1000)
  return date.toLocaleDateString('zh-CN')
}

// 安全截取标题 (处理中文)
function truncateTitle(title: string, maxLen: number): string {
  if (title.length > maxLen) {
    return title.slice(0, maxLen) + '...'
  }
  return title
}

// 搜索
async function handleSearch() {
  if (!searchKeyword.value.trim()) {
    message.warning('请输入搜索关键词')
    return
  }
  try {
    await searchStore.search(searchKeyword.value)
  } catch (error) {
    message.error(String(error))
  }
}

// 翻页
async function handlePageChange(page: number) {
  try {
    await searchStore.setPage(page)
  } catch (error) {
    message.error(String(error))
  }
}

// 排序变化
async function handleOrderChange(order: string) {
  try {
    await searchStore.setOrder(order as any)
  } catch (error) {
    message.error(String(error))
  }
}

// 打开批量评论弹窗
function openBatchComment() {
  if (searchStore.selectedCount === 0) {
    message.warning('请先选择要评论的视频')
    return
  }
  commentTarget.value = 'batch'
  singleVideo.value = null
  commentContent.value = ''
  selectedTemplate.value = null
  showCommentModal.value = true
}

// 打开单视频快速评论
function openSingleComment(video: VideoItem) {
  commentTarget.value = 'single'
  singleVideo.value = video
  commentContent.value = ''
  selectedTemplate.value = null
  showCommentModal.value = true
}

// 选择模板
function handleTemplateSelect(templateId: string) {
  selectedTemplate.value = templateId
  const template = templateStore.templates.find((t) => t.id === templateId)
  if (template) {
    commentContent.value = template.content
  }
}

// 提交评论
async function submitComment() {
  if (!commentContent.value.trim()) {
    message.warning('请输入评论内容')
    return
  }

  if (commentTarget.value === 'single' && singleVideo.value) {
    // 单视频评论
    try {
      const result = await commentStore.sendComment(
        singleVideo.value,
        commentContent.value
      )
      if (result.success) {
        message.success('评论发送成功')
        showCommentModal.value = false
      } else {
        message.error(result.error_msg || '评论发送失败')
      }
    } catch (error) {
      message.error(String(error))
    }
  } else {
    // 批量评论
    const videos = searchStore.selectedVideoList
    if (videos.length === 0) {
      message.warning('没有选中的视频')
      return
    }

    dialog.warning({
      title: '确认批量评论',
      content: `将对 ${videos.length} 个视频发送评论，每条间隔 ${commentStore.commentInterval} 秒，预计需要 ${Math.ceil((videos.length * commentStore.commentInterval) / 60)} 分钟，确定继续？`,
      positiveText: '确定',
      negativeText: '取消',
      onPositiveClick: async () => {
        try {
          await commentStore.batchSendComments(videos, commentContent.value)
          message.success('批量评论任务已启动')
          showCommentModal.value = false
        } catch (error) {
          message.error(String(error))
        }
      },
    })
  }
}

// 用户菜单操作
function handleUserMenuSelect(key: string) {
  if (key === 'templates') {
    router.push('/templates')
  } else if (key === 'logout') {
    dialog.warning({
      title: '退出登录',
      content: '确定要退出登录吗？',
      positiveText: '确定',
      negativeText: '取消',
      onPositiveClick: async () => {
        await authStore.logout()
        router.push('/login')
      },
    })
  }
}

// 模板选项
const templateOptions = computed(() =>
  templateStore.templates.map((t) => ({
    label: t.name,
    value: t.id,
  }))
)

onMounted(async () => {
  // 检查登录状态
  const isLoggedIn = await authStore.checkLogin()
  if (!isLoggedIn) {
    router.push('/login')
    return
  }

  await authStore.fetchUserInfo()
  await templateStore.fetchTemplates()
  await commentStore.fetchCommentInterval()
})
</script>

<template>
  <NLayout class="main-layout">
    <!-- 顶部导航 -->
    <NLayoutHeader class="header" bordered>
      <div class="header-content">
        <div class="logo">B站评论助手</div>

        <div class="header-right">
          <!-- 批量任务进度 -->
          <NBadge
            v-if="commentStore.batchStatus"
            :value="commentStore.progress + '%'"
            :type="commentStore.isRunning ? 'info' : 'success'"
          >
            <NButton size="small" quaternary>
              任务: {{ commentStore.batchStatus.completed }}/{{
                commentStore.batchStatus.total
              }}
            </NButton>
          </NBadge>

          <!-- 用户信息 -->
          <NDropdown
            v-if="authStore.user"
            :options="userMenuOptions"
            @select="handleUserMenuSelect"
          >
            <NSpace align="center" class="user-info" style="cursor: pointer">
              <NAvatar :src="authStore.user.face" round size="small" />
              <span>{{ authStore.user.uname }}</span>
            </NSpace>
          </NDropdown>
        </div>
      </div>
    </NLayoutHeader>

    <!-- 主内容区 -->
    <NLayoutContent class="content">
      <!-- 搜索栏 -->
      <div class="search-bar">
        <NSpace>
          <NInput
            v-model:value="searchKeyword"
            placeholder="输入关键词搜索视频"
            style="width: 400px"
            clearable
            @keyup.enter="handleSearch"
          />
          <NButton type="primary" @click="handleSearch">搜索</NButton>
        </NSpace>

        <NSpace>
          <NSelect
            :value="searchStore.order"
            :options="orderOptions"
            style="width: 120px"
            @update:value="handleOrderChange"
          />
          <NButton
            v-if="searchStore.results.length > 0"
            @click="searchStore.selectAll"
          >
            全选
          </NButton>
          <NButton
            v-if="searchStore.selectedCount > 0"
            type="info"
            @click="searchStore.clearSelection"
          >
            清除选择 ({{ searchStore.selectedCount }})
          </NButton>
          <NButton type="primary" :disabled="searchStore.selectedCount === 0" @click="openBatchComment">
            批量评论
          </NButton>
        </NSpace>
      </div>

      <!-- 加载状态 -->
      <div v-if="searchStore.isLoading" class="loading">
        <NSpin size="large" />
      </div>

      <!-- 搜索结果 -->
      <div v-else-if="searchStore.results.length > 0" class="video-list">
        <NCard
          v-for="video in searchStore.results"
          :key="video.bvid"
          class="video-card"
          :class="{ selected: searchStore.isSelected(video.bvid) }"
        >
          <div class="video-item">
            <NCheckbox
              :checked="searchStore.isSelected(video.bvid)"
              @update:checked="searchStore.toggleSelect(video.bvid)"
            />

            <NImage
              :src="video.pic"
              width="160"
              height="100"
              object-fit="cover"
              lazy
              preview-disabled
              class="video-cover"
            />

            <div class="video-info">
              <div class="video-title" :title="video.title">
                {{ video.title }}
              </div>
              <div class="video-meta">
                <span>{{ video.author }}</span>
                <span>{{ formatPlay(video.play) }}播放</span>
                <span>{{ video.duration }}</span>
                <span>{{ formatDate(video.pubdate) }}</span>
              </div>
              <div class="video-desc">{{ video.description }}</div>
            </div>

            <NButton size="small" type="primary" ghost @click="openSingleComment(video)">
              快速评论
            </NButton>
          </div>
        </NCard>

        <!-- 分页 -->
        <div class="pagination">
          <NPagination
            :page="searchStore.page"
            :page-count="searchStore.totalPages"
            :page-size="searchStore.pageSize"
            @update:page="handlePageChange"
          />
        </div>
      </div>

      <!-- 空状态 -->
      <div v-else class="empty">
        <NEmpty description="输入关键词搜索视频" />
      </div>
    </NLayoutContent>

    <!-- 评论弹窗 -->
    <NModal
      v-model:show="showCommentModal"
      preset="card"
      :title="commentTarget === 'single' ? '快速评论' : `批量评论 (${searchStore.selectedCount}个视频)`"
      style="width: 500px"
    >
      <NSpace vertical>
        <!-- 单视频信息 -->
        <div v-if="singleVideo" class="comment-video-info">
          <NImage
            :src="singleVideo.pic"
            width="120"
            height="75"
            object-fit="cover"
            preview-disabled
          />
          <div>
            <div class="video-title">{{ singleVideo.title }}</div>
            <div class="video-meta">{{ singleVideo.author }}</div>
          </div>
        </div>

        <!-- 模板选择 -->
        <NSelect
          v-if="templateOptions.length > 0"
          v-model:value="selectedTemplate"
          :options="templateOptions"
          placeholder="选择评论模板"
          clearable
          @update:value="handleTemplateSelect"
        />

        <!-- 评论内容 -->
        <NInput
          v-model:value="commentContent"
          type="textarea"
          placeholder="输入评论内容"
          :rows="4"
        />

        <!-- 提示 -->
        <div v-if="commentTarget === 'batch'" class="comment-tip">
          <NTag type="warning">
            批量评论将按 {{ commentStore.commentInterval }} 秒/条 的间隔发送
          </NTag>
        </div>

        <!-- 按钮 -->
        <NSpace justify="end">
          <NButton @click="showCommentModal = false">取消</NButton>
          <NButton type="primary" :loading="commentStore.isLoading" @click="submitComment">
            发送评论
          </NButton>
        </NSpace>
      </NSpace>
    </NModal>

    <!-- 任务进度浮窗 -->
    <div v-if="commentStore.batchStatus" class="task-progress">
      <NCard title="评论任务" size="small">
        <template #header-extra>
          <NSpace>
            <NButton v-if="commentStore.isRunning" size="tiny" type="error" @click="commentStore.cancelBatch">
              取消
            </NButton>
            <NButton size="tiny" type="default" @click="commentStore.clearBatch">
              关闭
            </NButton>
          </NSpace>
        </template>

        <div class="task-list">
          <div
            v-for="task in commentStore.batchStatus.tasks"
            :key="task.id"
            class="task-item"
          >
            <span class="task-title">{{ truncateTitle(task.video.title, 20) }}</span>
            <NTag
              :type="
                task.status === 'success'
                  ? 'success'
                  : task.status === 'failed'
                    ? 'error'
                    : task.status === 'running'
                      ? 'info'
                      : 'default'
              "
              size="small"
            >
              {{
                task.status === 'success'
                  ? '成功'
                  : task.status === 'failed'
                    ? '失败'
                    : task.status === 'running'
                      ? '发送中'
                      : '等待'
              }}
            </NTag>
          </div>
        </div>

        <div class="task-summary">
          <span v-if="commentStore.isRunning">
            进度: {{ commentStore.batchStatus.completed }}/{{ commentStore.batchStatus.total }}
          </span>
          <span v-else>
            已完成
          </span>
          | 成功: {{ commentStore.batchStatus.success }} | 失败:
          {{ commentStore.batchStatus.failed }}
        </div>
      </NCard>
    </div>
  </NLayout>
</template>

<style scoped>
.main-layout {
  height: 100vh;
}

.header {
  padding: 0 24px;
  height: 60px;
  display: flex;
  align-items: center;
}

.header-content {
  width: 100%;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.logo {
  font-size: 18px;
  font-weight: bold;
  color: #fb7299;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.content {
  padding: 24px;
  height: calc(100vh - 60px);
  overflow-y: auto;
}

.search-bar {
  display: flex;
  justify-content: space-between;
  margin-bottom: 24px;
}

.loading {
  display: flex;
  justify-content: center;
  padding: 100px 0;
}

.empty {
  display: flex;
  justify-content: center;
  padding: 100px 0;
}

.video-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.video-card {
  transition: all 0.2s;
}

.video-card.selected {
  border-color: #18a058;
  background: rgba(24, 160, 88, 0.1);
}

.video-item {
  display: flex;
  align-items: center;
  gap: 16px;
}

.video-cover {
  border-radius: 4px;
  flex-shrink: 0;
}

.video-info {
  flex: 1;
  min-width: 0;
}

.video-title {
  font-size: 16px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-bottom: 8px;
}

.video-meta {
  display: flex;
  gap: 16px;
  color: #999;
  font-size: 13px;
  margin-bottom: 8px;
}

.video-desc {
  color: #666;
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pagination {
  display: flex;
  justify-content: center;
  margin-top: 24px;
}

.comment-video-info {
  display: flex;
  gap: 12px;
  padding: 12px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 8px;
}

.comment-tip {
  margin-top: 8px;
}

.task-progress {
  position: fixed;
  right: 24px;
  bottom: 24px;
  width: 360px;
  z-index: 100;
}

.task-list {
  max-height: 300px;
  overflow-y: auto;
  padding-right: 8px;
}

.task-list::-webkit-scrollbar {
  width: 6px;
}

.task-list::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 3px;
}

.task-list::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.3);
}

.task-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  gap: 12px;
}

.task-title {
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  min-width: 0;
}

.task-summary {
  margin-top: 12px;
  font-size: 12px;
  color: #999;
}
</style>
