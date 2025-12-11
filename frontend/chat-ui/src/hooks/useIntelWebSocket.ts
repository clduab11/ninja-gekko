import { useEffect, useState, useRef } from 'react';
import { IntelItem } from '../services/api';

const WS_URL = 'ws://localhost:8787/ws';

export function useIntelWebSocket() {
  const [items, setItems] = useState<IntelItem[]>([]);
  const [isConnected, setIsConnected] = useState(false);
  const socketRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    // Create WebSocket connection
    const socket = new WebSocket(WS_URL);
    socketRef.current = socket;

    socket.onopen = () => {
      console.log('Intel Stream connected');
      setIsConnected(true);
      
      // Subscribe to intel stream
      socket.send(JSON.stringify({
        type: 'subscribe',
        subscriptions: ['intel_stream']
      }));
    };

    socket.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data);
        
        if (message.type === 'intel_update') {
          const newItem = message.item as IntelItem;
          // Add new item to top of list
          setItems(prevItems => {
            // Check for duplicates
            if (prevItems.some(item => item.id === newItem.id)) {
              return prevItems;
            }
            return [newItem, ...prevItems].slice(0, 50); // Keep last 50
          });
        }
      } catch (err) {
        console.error('Failed to parse websocket message:', err);
      }
    };

    socket.onclose = () => {
      console.log('Intel Stream disconnected');
      setIsConnected(false);
    };

    return () => {
      if (socketRef.current) {
        socketRef.current.close();
      }
    };
  }, []);

  return { items, isConnected };
}
