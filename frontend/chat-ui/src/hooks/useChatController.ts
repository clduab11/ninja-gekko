import { useCallback, useEffect, useState } from 'react';
import { useMutation, useQuery } from '@tanstack/react-query';

import { useChatStore } from '../state/chatStore';
import { usePersonaStore } from '../state/personaStore';
import { fetchChatHistory, fetchPersona, sendChatMessage, updatePersona } from '../services/api';
import { ChatMessage, PersonaSettings } from '../types';

export function useChatController() {
  const { messages, setMessages, appendMessage, diagnostics, setDiagnostics, selectedModel } = useChatStore();
  const { persona, setPersona } = usePersonaStore();
  const [isPersonaLoading, setPersonaLoading] = useState(false);
  const [isSending, setIsSending] = useState(false);

  // ... (Queries kept same) ... 
  
  // Replace mutation with streaming handler
  const sendMessage = useCallback(async (prompt: string) => {
      setIsSending(true);

      // 1. Optimistic User Message
      const userMsg: ChatMessage = {
        id: crypto.randomUUID(),
        role: 'user',
        content: prompt,
        timestamp: new Date().toISOString()
      };
      appendMessage(userMsg);

      // 2. Optimistic Assistant Message (Empty placeholder)
      const assistantId = crypto.randomUUID();
      const assistantMsg: ChatMessage = {
        id: assistantId,
        role: 'assistant',
        content: '', // Start empty
        timestamp: new Date().toISOString()
      };
      appendMessage(assistantMsg);

      try {
          // 3. Prepare context window (last 10 messages)
          const context = [...messages, userMsg].slice(-10);
          // Add system prompt based on persona if needed? For now just context.
          
          let fullContent = '';
          
          // 4. Stream response
          const { streamChat } = await import('../services/api'); // Dynamic import to avoid cycles if any
          
          for await (const chunk of streamChat(selectedModel, context)) {
              fullContent += chunk;
              
              // Update store reference directly or via setMessages 
              // (Zustand immutable update pattern needed)
              useChatStore.setState(state => ({
                  messages: state.messages.map(m => 
                      m.id === assistantId ? { ...m, content: fullContent } : m
                  )
              }));
          }

          // 5. Persist the interaction to the backend
          const { persistChatInteraction } = await import('../services/api');
          const finalAssistantMsg = { ...assistantMsg, content: fullContent };
          
          // Fire and forget, or await? Await to ensure it's saved.
          await persistChatInteraction(userMsg, finalAssistantMsg);

      } catch (e) {
          console.error("Chat streaming failed", e);
          useChatStore.setState(state => ({
             messages: [...state.messages, {
                 id: crypto.randomUUID(),
                 role: 'system',
                 content: `Error: ${e instanceof Error ? e.message : 'Unknown error'}`,
                 timestamp: new Date().toISOString()
             }]
          }));
      } finally {
          setIsSending(false);
      }
  }, [messages, selectedModel, appendMessage]);

  const savePersona = useCallback(async (settings: PersonaSettings) => {
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
    isSending,
    isPersonaLoading,
    sendMessage,
    savePersona
  };
}
