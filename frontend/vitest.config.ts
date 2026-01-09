import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'
import { fileURLToPath } from 'node:url'

export default defineConfig({
    plugins: [vue()],
    test: {
        environment: 'happy-dom',
        globals: true,
        setupFiles: ['./src/test/setup.ts'],
        coverage: {
            provider: 'istanbul',
            reporter: ['text', 'html'],
            include: [
                'src/components/**/*.vue',
                'src/stores/**/*.ts',
                'src/views/**/*.vue',
            ],
            exclude: [
                'src/**/*.spec.ts',
                'src/**/*.test.ts',
                'node_modules',
            ],
        },
    },
    resolve: {
        alias: {
            '@': fileURLToPath(new URL('./src', import.meta.url)),
        },
    },
})
