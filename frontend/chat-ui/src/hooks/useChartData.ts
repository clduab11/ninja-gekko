import { useState, useEffect, useRef } from 'react';
import { useQuery } from '@tanstack/react-query';
import { fetchHistoricalCandles } from '../services/api';
import { MarketDataPoint } from '../types';

export function useChartData(symbol: string, timeframe: string) {
    const [liveData, setLiveData] = useState<MarketDataPoint[]>([]);
    
    // Fetch initial historical data
    const { data: historicalData, isLoading, error } = useQuery({ 
        queryKey: ['market-history', symbol, timeframe], 
        queryFn: () => fetchHistoricalCandles(symbol, timeframe),
        refetchOnWindowFocus: false
    });

    // Merge historical and live data
    // In a real app, we'd manage the merging more carefully to avoid duplicates
    // For now, we'll just return historicalData and append live updates if we had them
    // But since we are replacing the simple WS logic, let's keep it simple first:
    // This hook primarily manages the historical fetch.
    
    // WebSocket logic for live updates
    useEffect(() => {
        if (!symbol) return;

        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        // Note: The backend /api/v1/ws expects params like ?symbols=BTC-USD
        const wsUrl = `${protocol}//${window.location.host}/api/v1/ws?symbols=${symbol}`;
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
            console.log(`Connected to market stream for ${symbol}`);
        };

        ws.onmessage = (event) => {
            try {
                const msg = JSON.parse(event.data);
                if (msg.type === 'market_data' && msg.data?.symbol === symbol) {
                   // In a full implementation, we would update the latest candle here
                   // For now, we will expose the latest price separately or try to update the chart
                   // Let's just expose the latest price for the header stats for now
                }
            } catch (e) {
                console.error('WS Parse Error', e);
            }
        };

        return () => {
            ws.close();
        };
    }, [symbol]);

    return {
        data: historicalData || [],
        isLoading,
        error
    };
}
