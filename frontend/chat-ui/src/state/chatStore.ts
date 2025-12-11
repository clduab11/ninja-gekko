import { create } from 'zustand';
import { ChatMessage, DiagnosticLog } from '../types';

interface ChatStore {
  messages: ChatMessage[];
  diagnostics: DiagnosticLog[];
  selectedModel: string;
  setMessages: (messages: ChatMessage[]) => void;
  appendMessage: (message: ChatMessage) => void;
  setDiagnostics: (diagnostics: DiagnosticLog[]) => void;
  setSelectedModel: (model: string) => void;
}

export const useChatStore = create<ChatStore>((set) => ({
  messages: [],
  diagnostics: [],
  selectedModel: 'nvidia/nemotron-nano-12b-v2-vl:free', // Default
  setMessages: (messages) => set({ messages }),
  appendMessage: (message) =>
    set((state) => ({
      messages: [...state.messages, message]
    })),
  setDiagnostics: (diagnostics) => set({ diagnostics }),
  setSelectedModel: (model) => set({ selectedModel: model })
}));
