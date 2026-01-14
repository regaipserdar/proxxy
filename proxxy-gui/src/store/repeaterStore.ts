import { create } from 'zustand';
import { apolloClient } from '@/graphql/client';
import {
    GET_REPEATER_TABS,
    CREATE_REPEATER_TAB,
    UPDATE_REPEATER_TAB,
    DELETE_REPEATER_TAB
} from '@/graphql/operations';

export interface RepeaterTask {
    id: string;
    name: string;
    request: string;
    response?: string;
    agentId?: string;
    targetUrl?: string;
    timestamp: number;
}

interface RepeaterState {
    tasks: RepeaterTask[];
    activeTaskId: string | null;
    isLoading: boolean;
    isSynced: boolean;
    loadTabs: () => Promise<void>;
    addTask: (task: Omit<RepeaterTask, 'id' | 'timestamp'>) => Promise<string>;
    updateTask: (id: string, updates: Partial<RepeaterTask>) => Promise<void>;
    removeTask: (id: string) => Promise<void>;
    setActiveTaskId: (id: string | null) => void;
}

// Helper to convert backend tab to local format
const backendToLocal = (tab: any): RepeaterTask => {
    const template = tab.requestTemplate || {};
    // Reconstruct raw HTTP request from template
    let request = `${template.method || 'GET'} ${template.url || '/'} HTTP/1.1\n`;
    if (template.headers) {
        try {
            const headers = typeof template.headers === 'string'
                ? JSON.parse(template.headers)
                : template.headers;
            Object.entries(headers).forEach(([k, v]) => {
                request += `${k}: ${v}\n`;
            });
        } catch (e) {
            console.error('[RepeaterStore] Failed to parse headers:', e);
        }
    }
    request += `\n${template.body || ''}`;

    return {
        id: tab.id,
        name: tab.name,
        request,
        agentId: tab.targetAgentId,
        timestamp: Date.now()
    };
};

export const useRepeaterStore = create<RepeaterState>()((set, get) => ({
    tasks: [],
    activeTaskId: localStorage.getItem('repeater-active-tab'),
    isLoading: false,
    isSynced: false,

    loadTabs: async () => {
        set({ isLoading: true });
        try {
            const { data } = await apolloClient.query({
                query: GET_REPEATER_TABS,
                fetchPolicy: 'network-only'
            });
            const tabs = (data?.repeaterTabs || []).map(backendToLocal);
            set({
                tasks: tabs,
                isSynced: true,
                activeTaskId: get().activeTaskId || tabs[0]?.id || null
            });
        } catch (e) {
            console.error('[RepeaterStore] Failed to load tabs:', e);
        } finally {
            set({ isLoading: false });
        }
    },

    addTask: async (task) => {
        // Optimistic ID for immediate UI feedback
        const tempId = `temp-${Date.now()}`;
        const tempTask: RepeaterTask = { ...task, id: tempId, timestamp: Date.now() };
        set((state) => ({
            tasks: [...state.tasks, tempTask],
            activeTaskId: tempId
        }));

        try {
            // Parse request to get template
            const lines = task.request.split('\n');
            const [method, path] = (lines[0] || 'GET /').split(' ');
            const hostLine = lines.find(l => l.toLowerCase().startsWith('host:'));
            const host = hostLine?.split(':').slice(1).join(':').trim() || 'example.com';
            const url = path?.startsWith('/') ? `http://${host}${path}` : (path || '/');

            const headers: Record<string, string> = {};
            let bodyStart = lines.findIndex(l => l === '');
            for (let i = 1; i < (bodyStart === -1 ? lines.length : bodyStart); i++) {
                const colonIdx = lines[i].indexOf(':');
                if (colonIdx !== -1) {
                    headers[lines[i].slice(0, colonIdx).trim()] = lines[i].slice(colonIdx + 1).trim();
                }
            }
            const body = bodyStart !== -1 ? lines.slice(bodyStart + 1).join('\n') : '';

            const { data } = await apolloClient.mutate({
                mutation: CREATE_REPEATER_TAB,
                variables: {
                    input: {
                        name: task.name,
                        requestTemplate: {
                            method: method || 'GET',
                            url,
                            headers: JSON.stringify(headers),
                            body
                        },
                        targetAgentId: task.agentId || null
                    }
                }
            });

            const serverId = data?.createRepeaterTab?.id;
            if (serverId) {
                // Replace temp task with server-confirmed task
                set((state) => ({
                    tasks: state.tasks.map(t => t.id === tempId ? { ...t, id: serverId } : t),
                    activeTaskId: state.activeTaskId === tempId ? serverId : state.activeTaskId
                }));
                localStorage.setItem('repeater-active-tab', serverId);
                return serverId;
            }
        } catch (e) {
            console.error('[RepeaterStore] Failed to create tab:', e);
            // Rollback
            set((state) => ({
                tasks: state.tasks.filter(t => t.id !== tempId),
                activeTaskId: state.activeTaskId === tempId ? null : state.activeTaskId
            }));
        }
        return tempId;
    },

    updateTask: async (id, updates) => {
        // Optimistic update
        set((state) => ({
            tasks: state.tasks.map(t => t.id === id ? { ...t, ...updates } : t)
        }));

        // Skip temp tasks
        if (id.startsWith('temp-')) return;

        try {
            const input: any = {};
            if (updates.name) input.name = updates.name;
            if (updates.agentId !== undefined) input.targetAgentId = updates.agentId;
            if (updates.request) {
                // Parse request to template
                const lines = updates.request.split('\n');
                const [method, path] = (lines[0] || 'GET /').split(' ');
                const hostLine = lines.find(l => l.toLowerCase().startsWith('host:'));
                const host = hostLine?.split(':').slice(1).join(':').trim() || 'example.com';
                const url = path?.startsWith('/') ? `http://${host}${path}` : (path || '/');

                const headers: Record<string, string> = {};
                let bodyStart = lines.findIndex(l => l === '');
                for (let i = 1; i < (bodyStart === -1 ? lines.length : bodyStart); i++) {
                    const colonIdx = lines[i].indexOf(':');
                    if (colonIdx !== -1) {
                        headers[lines[i].slice(0, colonIdx).trim()] = lines[i].slice(colonIdx + 1).trim();
                    }
                }
                const body = bodyStart !== -1 ? lines.slice(bodyStart + 1).join('\n') : '';

                input.requestTemplate = {
                    method: method || 'GET',
                    url,
                    headers: JSON.stringify(headers),
                    body
                };
            }

            if (Object.keys(input).length > 0) {
                await apolloClient.mutate({
                    mutation: UPDATE_REPEATER_TAB,
                    variables: { id, input }
                });
            }
        } catch (e) {
            console.error('[RepeaterStore] Failed to update tab:', e);
        }
    },

    removeTask: async (id) => {
        // Optimistic delete
        const prevTasks = get().tasks;
        set((state) => ({
            tasks: state.tasks.filter(t => t.id !== id),
            activeTaskId: state.activeTaskId === id ? (state.tasks[0]?.id || null) : state.activeTaskId
        }));

        if (id.startsWith('temp-')) return;

        try {
            await apolloClient.mutate({
                mutation: DELETE_REPEATER_TAB,
                variables: { id }
            });
        } catch (e) {
            console.error('[RepeaterStore] Failed to delete tab:', e);
            // Rollback
            set({ tasks: prevTasks });
        }
    },

    setActiveTaskId: (id) => {
        set({ activeTaskId: id });
        if (id) localStorage.setItem('repeater-active-tab', id);
    }
}));
