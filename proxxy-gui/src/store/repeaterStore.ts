import { create } from 'zustand';
import { persist } from 'zustand/middleware';

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
    addTask: (task: Omit<RepeaterTask, 'id' | 'timestamp'>) => string;
    updateTask: (id: string, updates: Partial<RepeaterTask>) => void;
    removeTask: (id: string) => void;
    setActiveTaskId: (id: string | null) => void;
}

export const useRepeaterStore = create<RepeaterState>()(
    persist(
        (set) => ({
            tasks: [],
            activeTaskId: null,
            addTask: (task) => {
                const id = Math.random().toString(36).substring(7);
                const newTask = { ...task, id, timestamp: Date.now() };
                set((state) => ({
                    tasks: [...state.tasks, newTask],
                    activeTaskId: id,
                }));
                return id;
            },
            updateTask: (id, updates) => set((state) => ({
                tasks: state.tasks.map((t) => (t.id === id ? { ...t, ...updates } : t)),
            })),
            removeTask: (id) => set((state) => ({
                tasks: state.tasks.filter((t) => t.id !== id),
                activeTaskId: state.activeTaskId === id ? null : state.activeTaskId,
            })),
            setActiveTaskId: (id) => set({ activeTaskId: id }),
        }),
        {
            name: 'proxxy-repeater-storage',
        }
    )
);
