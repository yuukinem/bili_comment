import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { CommentTemplate } from '~/types/bilibili'

export const useTemplateStore = defineStore('template', {
  state: () => ({
    templates: [] as CommentTemplate[],
    isLoading: false,
  }),

  actions: {
    // 加载所有模板
    async fetchTemplates() {
      try {
        this.isLoading = true
        this.templates = await invoke<CommentTemplate[]>('get_templates')
      } catch (error) {
        console.error('加载模板失败:', error)
        throw error
      } finally {
        this.isLoading = false
      }
    },

    // 创建模板
    async createTemplate(name: string, content: string) {
      try {
        const template = await invoke<CommentTemplate>('create_template', {
          name,
          content,
        })
        this.templates.push(template)
        return template
      } catch (error) {
        console.error('创建模板失败:', error)
        throw error
      }
    },

    // 更新模板
    async updateTemplate(id: string, name: string, content: string) {
      try {
        const template = await invoke<CommentTemplate>('update_template', {
          id,
          name,
          content,
        })
        const index = this.templates.findIndex((t) => t.id === id)
        if (index !== -1) {
          this.templates[index] = template
        }
        return template
      } catch (error) {
        console.error('更新模板失败:', error)
        throw error
      }
    },

    // 删除模板
    async deleteTemplate(id: string) {
      try {
        await invoke('delete_template', { id })
        this.templates = this.templates.filter((t) => t.id !== id)
      } catch (error) {
        console.error('删除模板失败:', error)
        throw error
      }
    },
  },
})
