import { create } from 'zustand';
import type { ServerId } from '../language-support';
export interface ServerInfo { id: ServerId; executable: string; available: boolean; path: string | null; install_program: string; install_args: string[] }
export const useLanguageStore = create<{
  open: boolean; revision: number; statuses: Partial<Record<ServerId, { state: string; detail?: string }>>;
  report: (id: ServerId, state: string, detail?: string) => void;
}>((set) => ({ open: false, revision: 0, statuses: {}, report: (id, state, detail) => set(s => ({ statuses: { ...s.statuses, [id]: { state, detail } } })) }));
