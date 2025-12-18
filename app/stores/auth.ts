import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { UserInfo, QrCodeData, LoginPollResult, LoginStatus } from '~/types/bilibili'

export const useAuthStore = defineStore('auth', {
  state: () => ({
    user: null as UserInfo | null,
    isLoading: false,
    qrCode: null as QrCodeData | null,
    loginStatus: 'waiting' as LoginStatus,
    loginMessage: '',
  }),

  getters: {
    isLoggedIn: (state) => state.user !== null && state.user.is_login,
  },

  actions: {
    // 获取用户信息
    async fetchUserInfo() {
      try {
        this.isLoading = true
        const user = await invoke<UserInfo | null>('get_user_info')
        this.user = user
        return user
      } catch (error) {
        console.error('获取用户信息失败:', error)
        this.user = null
        return null
      } finally {
        this.isLoading = false
      }
    },

    // 获取登录二维码
    async getLoginQrCode() {
      try {
        this.isLoading = true
        this.loginStatus = 'waiting'
        this.loginMessage = '请使用B站APP扫码登录'
        const qrCode = await invoke<QrCodeData>('get_login_qrcode')
        this.qrCode = qrCode
        return qrCode
      } catch (error) {
        console.error('获取二维码失败:', error)
        this.loginStatus = 'error'
        this.loginMessage = String(error)
        throw error
      } finally {
        this.isLoading = false
      }
    },

    // 轮询登录状态
    async pollLoginStatus() {
      if (!this.qrCode) return

      try {
        const result = await invoke<LoginPollResult>('poll_login_status', {
          qrcodeKey: this.qrCode.qrcode_key,
        })

        this.loginStatus = result.status
        this.loginMessage = result.message

        if (result.status === 'confirmed') {
          // 登录成功，获取用户信息
          await this.fetchUserInfo()
        }

        return result
      } catch (error) {
        console.error('轮询登录状态失败:', error)
        this.loginStatus = 'error'
        this.loginMessage = String(error)
        throw error
      }
    },

    // 退出登录
    async logout() {
      try {
        await invoke('logout')
        this.user = null
        this.qrCode = null
        this.loginStatus = 'waiting'
        this.loginMessage = ''
      } catch (error) {
        console.error('退出登录失败:', error)
        throw error
      }
    },

    // 检查登录状态
    async checkLogin() {
      try {
        const isValid = await invoke<boolean>('check_login_valid')
        if (!isValid) {
          this.user = null
        }
        return isValid
      } catch {
        this.user = null
        return false
      }
    },
  },
})
