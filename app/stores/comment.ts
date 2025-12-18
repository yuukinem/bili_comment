import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { VideoItem, BatchStatus, CommentResult } from '~/types/bilibili'

// 轮询定时器
let pollTimer: ReturnType<typeof setTimeout> | null = null
let pollFailCount = 0
const MAX_POLL_FAILS = 5

export const useCommentStore = defineStore('comment', {
  state: () => ({
    currentBatchId: null as string | null,
    batchStatus: null as BatchStatus | null,
    isLoading: false,
    commentInterval: 5, // 评论间隔秒数
    isPolling: false, // 是否正在轮询
  }),

  getters: {
    isRunning: (state) => {
      if (!state.batchStatus) return false
      if (!state.isPolling) return false
      return state.batchStatus.completed < state.batchStatus.total
    },
    progress: (state) => {
      if (!state.batchStatus || state.batchStatus.total === 0) return 0
      return Math.round((state.batchStatus.completed / state.batchStatus.total) * 100)
    },
    // 任务是否已完成（不管成功还是失败）
    isCompleted: (state) => {
      if (!state.batchStatus) return false
      return state.batchStatus.completed >= state.batchStatus.total
    },
  },

  actions: {
    // 获取评论间隔时间
    async fetchCommentInterval() {
      try {
        this.commentInterval = await invoke<number>('get_comment_interval')
      } catch (error) {
        console.error('获取评论间隔失败:', error)
      }
    },

    // 发送单条评论
    async sendComment(video: VideoItem, content: string) {
      try {
        this.isLoading = true
        const result = await invoke<CommentResult>('send_comment', {
          bvid: video.bvid,
          aid: video.aid,
          content,
        })
        return result
      } catch (error) {
        console.error('发送评论失败:', error)
        throw error
      } finally {
        this.isLoading = false
      }
    },

    // 批量发送评论
    async batchSendComments(videos: VideoItem[], content: string) {
      // 先清理之前的任务
      this.stopPolling()
      this.currentBatchId = null
      this.batchStatus = null
      pollFailCount = 0

      try {
        this.isLoading = true
        const batchId = await invoke<string>('batch_send_comments', {
          videos,
          content,
        })
        this.currentBatchId = batchId
        // 开始轮询状态
        this.startPolling()
        return batchId
      } catch (error) {
        console.error('启动批量评论失败:', error)
        throw error
      } finally {
        this.isLoading = false
      }
    },

    // 获取批量任务状态
    async fetchBatchStatus() {
      if (!this.currentBatchId) return null

      try {
        const status = await invoke<BatchStatus>('get_batch_status', {
          batchId: this.currentBatchId,
        })
        this.batchStatus = status
        pollFailCount = 0 // 成功后重置失败计数
        return status
      } catch (error) {
        console.error('获取批量状态失败:', error)
        pollFailCount++
        return null
      }
    },

    // 开始轮询状态
    startPolling() {
      this.stopPolling() // 先停止之前的轮询
      this.isPolling = true
      pollFailCount = 0

      const poll = async () => {
        if (!this.isPolling) return

        const status = await this.fetchBatchStatus()

        // 检查是否失败次数过多
        if (pollFailCount >= MAX_POLL_FAILS) {
          console.error('轮询失败次数过多，停止轮询')
          this.stopPolling()
          return
        }

        // 如果任务还在进行中，继续轮询
        if (status && status.completed < status.total) {
          pollTimer = setTimeout(poll, 1000)
        } else {
          // 任务完成，停止轮询
          this.isPolling = false
        }
      }
      poll()
    },

    // 停止轮询
    stopPolling() {
      this.isPolling = false
      if (pollTimer) {
        clearTimeout(pollTimer)
        pollTimer = null
      }
    },

    // 取消批量任务
    async cancelBatch() {
      if (!this.currentBatchId) return

      try {
        await invoke('cancel_batch', { batchId: this.currentBatchId })
        // 停止轮询并清理状态
        this.stopPolling()
      } catch (error) {
        console.error('取消批量任务失败:', error)
        throw error
      }
    },

    // 清理批量任务
    async clearBatch() {
      this.stopPolling()

      if (this.currentBatchId) {
        try {
          await invoke('clear_batch', { batchId: this.currentBatchId })
        } catch (error) {
          console.error('清理批量任务失败:', error)
        }
      }

      this.currentBatchId = null
      this.batchStatus = null
    },

    // 强制重置状态（用于出错后恢复）
    forceReset() {
      this.stopPolling()
      this.currentBatchId = null
      this.batchStatus = null
      this.isLoading = false
      pollFailCount = 0
    },
  },
})
