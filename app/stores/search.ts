import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { VideoItem, SearchResult, SearchOrder } from '~/types/bilibili'

// 防抖定时器
let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null

export const useSearchStore = defineStore('search', {
  state: () => ({
    keyword: '',
    results: [] as VideoItem[],
    page: 1,
    pageSize: 20,
    total: 0,
    order: 'totalrank' as SearchOrder,
    isLoading: false,
    selectedVideos: new Set<string>(), // 使用 bvid 作为 key
  }),

  getters: {
    totalPages: (state) => Math.ceil(state.total / state.pageSize),
    selectedCount: (state) => state.selectedVideos.size,
    selectedVideoList: (state) => {
      return state.results.filter((v) => state.selectedVideos.has(v.bvid))
    },
  },

  actions: {
    // 搜索视频
    async search(keyword?: string) {
      if (keyword !== undefined) {
        this.keyword = keyword
        this.page = 1
        this.selectedVideos.clear()
      }

      if (!this.keyword.trim()) {
        this.results = []
        this.total = 0
        return
      }

      // 防止重复请求
      if (this.isLoading) {
        console.log('搜索进行中，跳过重复请求')
        return
      }

      try {
        this.isLoading = true
        const result = await invoke<SearchResult>('search_videos', {
          keyword: this.keyword,
          page: this.page,
          pageSize: this.pageSize,
          order: this.order,
        })

        this.results = result.items
        this.total = result.total
        this.page = result.page
      } catch (error) {
        console.error('搜索失败:', error)
        throw error
      } finally {
        this.isLoading = false
      }
    },

    // 防抖搜索 (用于输入框实时搜索)
    searchDebounced(keyword: string, delay: number = 300) {
      if (searchDebounceTimer) {
        clearTimeout(searchDebounceTimer)
      }
      searchDebounceTimer = setTimeout(() => {
        this.search(keyword)
      }, delay)
    },

    // 切换页码
    async setPage(page: number) {
      this.page = page
      await this.search()
    },

    // 设置排序方式
    async setOrder(order: SearchOrder) {
      this.order = order
      this.page = 1
      await this.search()
    },

    // 切换视频选中状态
    toggleSelect(bvid: string) {
      if (this.selectedVideos.has(bvid)) {
        this.selectedVideos.delete(bvid)
      } else {
        this.selectedVideos.add(bvid)
      }
    },

    // 全选当前页
    selectAll() {
      this.results.forEach((v) => this.selectedVideos.add(v.bvid))
    },

    // 取消全选
    clearSelection() {
      this.selectedVideos.clear()
    },

    // 判断是否选中
    isSelected(bvid: string) {
      return this.selectedVideos.has(bvid)
    },
  },
})
