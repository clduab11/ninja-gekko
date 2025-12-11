import { useQuery } from '@tanstack/react-query';
import { fetchAggregateAccount } from '../../services/api';
import { DollarSign, TrendingUp, AlertTriangle } from 'lucide-react';

import ModelSelector from './ModelSelector';

const HeaderMetrics = () => {
    const { data: aggregateAccount, isLoading } = useQuery({ 
        queryKey: ['aggregate-account'], 
        queryFn: fetchAggregateAccount,
        refetchInterval: 5000 
    });

    if (isLoading || !aggregateAccount) {
        return (
            <div className="flex items-center gap-4 text-xs font-mono">
                <div className="flex flex-col gap-0.5 animate-pulse">
                    <span className="text-[9px] uppercase tracking-wider text-slate-500">Net Liquidity</span>
                    <span className="text-slate-400">---</span>
                </div>
            </div>
        );
    }

    const exposurePercentage = aggregateAccount.total_net_liquidity > 0 
        ? (aggregateAccount.total_exposure / aggregateAccount.total_net_liquidity) * 100 
        : 0;

    return (
        <div className="flex items-center gap-6 px-4 py-1 rounded bg-white/[0.02] border border-white/5 mx-6">
            <ModelSelector />
            <div className="h-6 w-px bg-white/10" />
            <div className="flex flex-col gap-0.5">
                <span className="text-[9px] uppercase tracking-wider text-slate-500 font-bold flex items-center gap-1">
                    <DollarSign className="h-2.5 w-2.5" /> Net Liq
                </span>
                <span className="text-sm font-bold text-white tracking-tight">
                    ${(aggregateAccount.total_net_liquidity ?? 0).toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                </span>
            </div>

            <div className="h-6 w-px bg-white/10" />

            <div className="flex flex-col gap-0.5">
                <span className="text-[9px] uppercase tracking-wider text-slate-500 font-bold flex items-center gap-1">
                    <TrendingUp className="h-2.5 w-2.5" /> Exposure
                </span>
                <div className="flex items-center gap-2">
                    <span className="text-sm font-bold text-slate-200 tracking-tight">
                        ${(aggregateAccount.total_exposure ?? 0).toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                    </span>
                    <span className={`text-[10px] font-medium px-1.5 py-0.5 rounded ${
                        exposurePercentage > 80 ? 'bg-red-500/20 text-red-400' : 
                        exposurePercentage > 50 ? 'bg-amber-500/20 text-amber-400' : 
                        'bg-emerald-500/20 text-emerald-400'
                    }`}>
                        {exposurePercentage.toFixed(1)}%
                    </span>
                </div>
            </div>

            {exposurePercentage > 80 && (
                <div className="flex items-center gap-1 text-red-500 text-[10px] font-bold uppercase tracking-wider animate-pulse ml-2">
                    <AlertTriangle className="h-3 w-3" />
                    <span>High Risk</span>
                </div>
            )}
        </div>
    );
};

export default HeaderMetrics;
