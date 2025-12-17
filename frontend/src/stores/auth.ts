import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import api from '@/api/client'

interface User {
    id: string
    email: string
    name: string | null
    avatar_url: string | null
}

export const useAuthStore = defineStore('auth', () => {
    const token = ref<string | null>(localStorage.getItem('token'))
    const user = ref<User | null>(null)

    const isAuthenticated = computed(() => !!token.value)

    function setToken(newToken: string) {
        token.value = newToken
        localStorage.setItem('token', newToken)
    }

    async function fetchUser() {
        if (!token.value) return null

        const response = await api.get('/auth/me')
        user.value = response.data
        return user.value
    }

    function logout() {
        token.value = null
        user.value = null
        localStorage.removeItem('token')
    }

    return {
        token,
        user,
        isAuthenticated,
        setToken,
        fetchUser,
        logout,
    }
})
