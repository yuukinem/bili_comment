<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import {
  NLayout,
  NLayoutHeader,
  NLayoutContent,
  NButton,
  NCard,
  NSpace,
  NInput,
  NModal,
  NEmpty,
  NPopconfirm,
  useMessage,
} from 'naive-ui'
import { useAuthStore } from '~/stores/auth'
import { useTemplateStore } from '~/stores/template'
import type { CommentTemplate } from '~/types/bilibili'

const router = useRouter()
const message = useMessage()
const authStore = useAuthStore()
const templateStore = useTemplateStore()

// 编辑弹窗
const showModal = ref(false)
const editingTemplate = ref<CommentTemplate | null>(null)
const templateName = ref('')
const templateContent = ref('')

// 格式化时间
function formatTime(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString('zh-CN')
}

// 打开新建弹窗
function openCreate() {
  editingTemplate.value = null
  templateName.value = ''
  templateContent.value = ''
  showModal.value = true
}

// 打开编辑弹窗
function openEdit(template: CommentTemplate) {
  editingTemplate.value = template
  templateName.value = template.name
  templateContent.value = template.content
  showModal.value = true
}

// 保存模板
async function saveTemplate() {
  if (!templateName.value.trim()) {
    message.warning('请输入模板名称')
    return
  }
  if (!templateContent.value.trim()) {
    message.warning('请输入模板内容')
    return
  }

  try {
    if (editingTemplate.value) {
      await templateStore.updateTemplate(
        editingTemplate.value.id,
        templateName.value,
        templateContent.value
      )
      message.success('模板更新成功')
    } else {
      await templateStore.createTemplate(templateName.value, templateContent.value)
      message.success('模板创建成功')
    }
    showModal.value = false
  } catch (error) {
    message.error(String(error))
  }
}

// 删除模板
async function deleteTemplate(id: string) {
  try {
    await templateStore.deleteTemplate(id)
    message.success('模板删除成功')
  } catch (error) {
    message.error(String(error))
  }
}

onMounted(async () => {
  // 检查登录状态
  const isLoggedIn = await authStore.checkLogin()
  if (!isLoggedIn) {
    router.push('/login')
    return
  }

  await templateStore.fetchTemplates()
})
</script>

<template>
  <NLayout class="main-layout">
    <!-- 顶部导航 -->
    <NLayoutHeader class="header" bordered>
      <div class="header-content">
        <NSpace align="center">
          <NButton text @click="router.push('/')">
            ← 返回
          </NButton>
          <span class="title">评论模板管理</span>
        </NSpace>

        <NButton type="primary" @click="openCreate">新建模板</NButton>
      </div>
    </NLayoutHeader>

    <!-- 主内容区 -->
    <NLayoutContent class="content">
      <div v-if="templateStore.templates.length > 0" class="template-list">
        <NCard
          v-for="template in templateStore.templates"
          :key="template.id"
          class="template-card"
        >
          <template #header>
            <span class="template-name">{{ template.name }}</span>
          </template>

          <template #header-extra>
            <NSpace>
              <NButton size="small" @click="openEdit(template)">编辑</NButton>
              <NPopconfirm @positive-click="deleteTemplate(template.id)">
                <template #trigger>
                  <NButton size="small" type="error">删除</NButton>
                </template>
                确定删除这个模板吗？
              </NPopconfirm>
            </NSpace>
          </template>

          <div class="template-content">{{ template.content }}</div>

          <div class="template-meta">
            创建于 {{ formatTime(template.created_at) }}
            <template v-if="template.updated_at !== template.created_at">
              | 更新于 {{ formatTime(template.updated_at) }}
            </template>
          </div>
        </NCard>
      </div>

      <div v-else class="empty">
        <NEmpty description="还没有评论模板">
          <template #extra>
            <NButton type="primary" @click="openCreate">创建第一个模板</NButton>
          </template>
        </NEmpty>
      </div>
    </NLayoutContent>

    <!-- 编辑弹窗 -->
    <NModal
      v-model:show="showModal"
      preset="card"
      :title="editingTemplate ? '编辑模板' : '新建模板'"
      style="width: 500px"
    >
      <NSpace vertical>
        <NInput
          v-model:value="templateName"
          placeholder="模板名称"
        />

        <NInput
          v-model:value="templateContent"
          type="textarea"
          placeholder="评论内容"
          :rows="6"
        />

        <NSpace justify="end">
          <NButton @click="showModal = false">取消</NButton>
          <NButton type="primary" @click="saveTemplate">保存</NButton>
        </NSpace>
      </NSpace>
    </NModal>
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

.title {
  font-size: 18px;
  font-weight: bold;
}

.content {
  padding: 24px;
  height: calc(100vh - 60px);
  overflow-y: auto;
}

.template-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
}

.template-card {
  height: fit-content;
}

.template-name {
  font-weight: 500;
}

.template-content {
  white-space: pre-wrap;
  word-break: break-all;
  color: #ccc;
  margin-bottom: 12px;
  max-height: 120px;
  overflow-y: auto;
}

.template-meta {
  font-size: 12px;
  color: #666;
}

.empty {
  display: flex;
  justify-content: center;
  padding: 100px 0;
}
</style>
