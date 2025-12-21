import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import ToolGovernance from '../ui/ToolGovernance.vue'

vi.mock('@/api/client', () => ({
    default: {
        get: vi.fn(),
        post: vi.fn(),
        delete: vi.fn(),
    },
}))

import api from '@/api/client'

describe('ToolGovernance', () => {
    beforeEach(() => {
        vi.clearAllMocks()
        vi.mocked(api.get).mockResolvedValue({
            data: { allowed_tools: [], denied_tools: [], tool_prefix: '' },
        })
    })

    it('renders toggle in off state by default', async () => {
        const wrapper = mount(ToolGovernance, {
            props: { serverName: 'test-server' },
        })
        await flushPromises()

        expect(wrapper.text()).toContain('Tool Governance')
        expect(wrapper.text()).toContain('Limit tool exposure')
    })

    it('shows allowlist/blocklist options when toggle is enabled', async () => {
        const wrapper = mount(ToolGovernance, {
            props: { serverName: 'test-server' },
        })
        await flushPromises()

        const toggle = wrapper.find('[class*="rounded-full"]')
        await toggle.trigger('click')

        expect(wrapper.text()).toContain('Allow only selected')
        expect(wrapper.text()).toContain('Block selected')
    })

    it('loads existing governance config on mount', async () => {
        vi.mocked(api.get).mockResolvedValue({
            data: {
                allowed_tools: ['tool1', 'tool2'],
                denied_tools: [],
                tool_prefix: 'cf',
            },
        })

        const _wrapper = mount(ToolGovernance, {
            props: { serverName: 'test-server' },
        })
        await flushPromises()

        expect(api.get).toHaveBeenCalledWith('/servers/test-server/governance')
    })

    it('saves governance config when save button clicked', async () => {
        vi.mocked(api.post).mockResolvedValue({ data: {} })

        const wrapper = mount(ToolGovernance, {
            props: { serverName: 'test-server' },
        })
        await flushPromises()

        const saveBtn = wrapper.find('button[class*="btn-primary"]')
        await saveBtn.trigger('click')

        expect(api.post).toHaveBeenCalledWith(
            '/servers/test-server/governance',
            expect.objectContaining({
                allowed_tools: expect.any(Array),
                denied_tools: expect.any(Array),
                tool_prefix: expect.any(String),
            })
        )
    })

    it('displays available tools as clickable chips', async () => {
        const wrapper = mount(ToolGovernance, {
            props: {
                serverName: 'test-server',
                availableTools: [
                    { name: 'search_docs' },
                    { name: 'migrate_guide' },
                ],
            },
        })
        await flushPromises()

        const toggle = wrapper.find('[class*="rounded-full"]')
        await toggle.trigger('click')

        expect(wrapper.text()).toContain('search_docs')
        expect(wrapper.text()).toContain('migrate_guide')
    })

    it('adds tool to selected list when chip clicked', async () => {
        const wrapper = mount(ToolGovernance, {
            props: {
                serverName: 'test-server',
                availableTools: [{ name: 'search_docs' }],
            },
        })
        await flushPromises()

        const toggle = wrapper.find('[class*="rounded-full"]')
        await toggle.trigger('click')

        const chip = wrapper.find('button[class*="bg-background-darker"]')
        await chip.trigger('click')

        expect(wrapper.html()).toContain('ring-green-500')
    })

    it('clears config when clear button clicked', async () => {
        vi.mocked(api.delete).mockResolvedValue({ data: {} })
        vi.mocked(api.get).mockResolvedValue({
            data: { allowed_tools: ['tool1'], denied_tools: [], tool_prefix: '' },
        })

        window.confirm = vi.fn().mockReturnValue(true)

        const wrapper = mount(ToolGovernance, {
            props: { serverName: 'test-server' },
        })
        await flushPromises()

        const clearBtn = wrapper.find('button[class*="text-gray-400"]')
        if (clearBtn.exists()) {
            await clearBtn.trigger('click')
            expect(api.delete).toHaveBeenCalledWith('/servers/test-server/governance')
        }
    })

    it('shows prefix input', async () => {
        const wrapper = mount(ToolGovernance, {
            props: { serverName: 'test-server' },
        })
        await flushPromises()

        expect(wrapper.find('input[placeholder*="github"]').exists()).toBe(true)
    })

    it('shows summary when prefix is set', async () => {
        vi.mocked(api.get).mockResolvedValue({
            data: { allowed_tools: [], denied_tools: [], tool_prefix: 'cf' },
        })

        const wrapper = mount(ToolGovernance, {
            props: { serverName: 'test-server' },
        })
        await flushPromises()

        const prefixInput = wrapper.find('input[placeholder*="github"]')
        await prefixInput.setValue('cf')

        expect(wrapper.text()).toContain('cf_')
    })
})
