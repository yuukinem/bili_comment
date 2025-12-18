// B站相关类型定义

export interface UserInfo {
  mid: number
  uname: string
  face: string
  is_login: boolean
}

export interface VideoItem {
  aid: number
  bvid: string
  title: string
  author: string
  mid: number
  pic: string
  play: number
  danmaku: number
  pubdate: number
  duration: string
  description: string
}

export interface SearchResult {
  page: number
  page_size: number
  total: number
  items: VideoItem[]
}

export interface CommentTemplate {
  id: string
  name: string
  content: string
  created_at: number
  updated_at: number
}

export type TaskStatus = 'pending' | 'running' | 'success' | 'failed' | 'cancelled'

export interface CommentTask {
  id: string
  video: VideoItem
  content: string
  status: TaskStatus
  error_msg?: string
  created_at: number
  completed_at?: number
}

export interface CommentResult {
  success: boolean
  rpid?: number
  error_msg?: string
}

export interface BatchStatus {
  batch_id: string
  total: number
  completed: number
  success: number
  failed: number
  tasks: CommentTask[]
}

export interface QrCodeData {
  url: string
  qrcode_key: string
  image_base64: string
}

export type LoginStatus = 'waiting' | 'scanned' | 'confirmed' | 'expired' | 'error'

export interface LoginPollResult {
  status: LoginStatus
  message: string
}

// 搜索排序选项
export type SearchOrder = 'totalrank' | 'click' | 'pubdate' | 'dm' | 'stow'

export const SEARCH_ORDER_OPTIONS = [
  { label: '综合排序', value: 'totalrank' },
  { label: '最多播放', value: 'click' },
  { label: '最新发布', value: 'pubdate' },
  { label: '最多弹幕', value: 'dm' },
  { label: '最多收藏', value: 'stow' },
] as const
