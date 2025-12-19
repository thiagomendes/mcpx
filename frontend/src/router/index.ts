import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = createRouter({
    history: createWebHistory(),
    routes: [
        {
            path: '/',
            name: 'home',
            component: () => import('@/views/Landing.vue'),
        },
        {
            path: '/login',
            name: 'login',
            component: () => import('@/views/SignIn.vue'),
        },
        {
            path: '/dashboard',
            name: 'dashboard',
            component: () => import('@/views/dashboard/Dashboard.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/servers',
            name: 'servers',
            component: () => import('@/views/dashboard/ServersList.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/servers/new',
            name: 'server-new',
            component: () => import('@/views/dashboard/ServerNew.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/servers/:name',
            name: 'server-details',
            component: () => import('@/views/dashboard/ServerDetails.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/oauth/callback',
            name: 'oauth-callback',
            component: () => import('@/views/OAuthCallback.vue'),
            // No auth required - popup handles its own flow
        },
    ],
})

// Navigation guard for protected routes
router.beforeEach((to, from, next) => {
    const authStore = useAuthStore()

    if (to.meta.requiresAuth && !authStore.isAuthenticated) {
        next('/login')
    } else {
        next()
    }
})

export default router
