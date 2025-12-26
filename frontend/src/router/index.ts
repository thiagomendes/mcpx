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
            path: '/join/:token',
            name: 'join-org',
            component: () => import('@/views/JoinOrg.vue'),
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
            path: '/gateways',
            name: 'gateways',
            component: () => import('@/views/dashboard/GatewaysList.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/gateways/:slug',
            name: 'gateway-details',
            component: () => import('@/views/dashboard/GatewayDetails.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/audit',
            name: 'audit',
            component: () => import('@/views/audit/AuditList.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/alerts',
            name: 'alerts',
            component: () => import('@/views/alerts/AlertsList.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/settings',
            name: 'settings',
            component: () => import('@/views/settings/Settings.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/settings/members',
            name: 'members',
            component: () => import('@/views/settings/Members.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/oauth/callback',
            name: 'oauth-callback',
            component: () => import('@/views/OAuthCallback.vue'),
        },
    ],
})

router.beforeEach(async (to, _from, next) => {
    const authStore = useAuthStore()

    // If user is authenticated but no user data loaded, fetch it
    if (authStore.isAuthenticated && !authStore.user) {
        await authStore.fetchUser()
    }

    if (to.meta.requiresAuth && !authStore.isAuthenticated) {
        next('/login')
    } else {
        next()
    }
})

export default router
