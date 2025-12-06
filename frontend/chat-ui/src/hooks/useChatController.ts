import { useCallback, useEffect, useState } from 'react';
import { useMutation, useQuery } from '@tanstack/react-query';

import { useChatStore } from '../state/chatStore';
import { usePersonaStore } from '../state/personaStore';
import { fetchChatHistory, fetchPersona, sendChatMessage, updatePersona } from '../services/api';
import { ChatMessage, PersonaSettings } from '../types';

export function useChatController() {
  const { messages, setMessages, appendMessage, diagnostics, setDiagnostics } = useChatStore();
  const { persona, setPersona } = usePersonaStore();
  const [isPersonaLoading, setPersonaLoading] = useState(false);

  useQuery({
    queryKey: ['chat-history'],
    queryFn: async () => {
      const history = await fetchChatHistory();
      setMessages(history);
      return history;
    }
  });

  useQuery({
    queryKey: ['persona'],
    queryFn: async () => {
      setPersonaLoading(true);
      const data = await fetchPersona();
      setPersona(data);
      setPersonaLoading(false);
      return data;
    }
  });

  const mutation = useMutation({
    mutationFn: async (prompt: string) => sendChatMessage(prompt),
    onSuccess: (payload) => {
      appendMessage(payload.reply);
      setPersona(payload.persona);
      setDiagnostics(payload.diagnostics);
    }
  });

  const sendMessage = useCallback(
    (prompt: string) => {
      const optimistic: ChatMessage = {
        id: crypto.randomUUID(),
        role: 'user',
        content: prompt,
        timestamp: new Date().toISOString()
      };
      appendMessage(optimistic);
      mutation.mutate(prompt);
    },
    [appendMessage, mutation]
  );

  const savePersona = useCallback(
    async (settings: PersonaSettings) => {
      setPersonaLoading(true);
      const result = await updatePersona(settings);
      setPersona(result);
      setPersonaLoading(false);
    },
    [setPersona]
  );

  useEffect(() => {
    if (persona && !messages.length) {
      setMessages([
        {
          id: crypto.randomUUID(),
          role: 'assistant',
          content: 'Gordon here. Feed me a prompt and we will tune strategy in real-time.',
          timestamp: new Date().toISOString()
        }
      ]);
    }
  }, [persona, messages.length, setMessages]);

  return {
    messages,
    persona,
    diagnostics,
    isSending: mutation.isPending,
    isPersonaLoading,
    sendMessage,
    savePersona
  };
}
