import { useEffect, useState, useRef } from 'react';
import { IntelItem, fetchIntelStream } from '../services/api';


export function useIntelWebSocket() {
  const [items, setItems] = useState<IntelItem[]>([]);
  const [isConnected, setIsConnected] = useState(false);
  const socketRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    // Fetch initial history
    fetchIntelStream(50).then(initialItems => {
        setItems(initialItems);
    }).catch(err => {
        console.error('Failed to fetch initial intel stream:', err);
    });

    // Determine WS protocol and host
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = window.location.host; // e.g. "localhost:5173" or "api.ninja-gekko.com"
    // Note: In development with Vite proxy, we might target the API port directly or use the proxy path.
    // If using the proxy setup in vite.config.ts (which proxies /api), we want:
    // ws://localhost:5173/api/v1/ws
    const wsUrl = `${protocol}//${host}/api/v1/ws`;
    
    // Create WebSocket connection
    const socket = new WebSocket(wsUrl);
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
