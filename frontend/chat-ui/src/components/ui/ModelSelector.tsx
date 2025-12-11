import { useState, useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import { BrainCircuit, Check, ChevronsUpDown, Cpu, GraduationCap, Zap, Code } from 'lucide-react';
import * as DropdownMenu from '@radix-ui/react-dropdown-menu';
import { fetchModels } from '../../services/api';
import { useChatStore } from '../../state/chatStore';

interface LlmModel {
  id: string;
  display_name: string;
  provider: string;
  context_window: number;
  specialization: string;
}

const ModelSelector = () => {
    const { selectedModel, setSelectedModel } = useChatStore();
    const [isOpen, setIsOpen] = useState(false);

    const { data: models = [] } = useQuery({
        queryKey: ['llm-models'],
        queryFn: fetchModels,
        staleTime: 1000 * 60 * 60 // 1 hour
    });

    const activeModel = models.find(m => m.id === selectedModel) || {
        display_name: 'Loading...',
        specialization: 'Initializing model registry...'
    };

    const getIcon = (spec: string) => {
        if (spec.includes('math') || spec.includes('quantitative')) return <GraduationCap className="w-4 h-4 text-blue-400" />;
        if (spec.includes('code') || spec.includes('coding')) return <Code className="w-4 h-4 text-purple-400" />;
        if (spec.includes('reasoning')) return <BrainCircuit className="w-4 h-4 text-amber-400" />;
        if (spec.includes('flash') || spec.includes('fast')) return <Zap className="w-4 h-4 text-emerald-400" />;
        return <Cpu className="w-4 h-4 text-slate-400" />;
    };

    return (
        <DropdownMenu.Root open={isOpen} onOpenChange={setIsOpen}>
            <DropdownMenu.Trigger asChild>
                <button 
                    className="flex items-center gap-2 px-3 py-1.5 rounded-md border border-white/10 bg-white/5 hover:bg-white/10 transition-colors outline-none"
                >
                    <div className="flex items-center gap-2">
                        {getIcon(activeModel.specialization || '')}
                        <div className="flex flex-col items-start gap-0.5">
                            <span className="text-xs font-bold text-slate-200">{activeModel.display_name}</span>
                            <span className="text-[9px] text-slate-500 max-w-[120px] truncate">{activeModel.specialization}</span>
                        </div>
                    </div>
                    <ChevronsUpDown className="w-3 h-3 text-slate-500 ml-2" />
                </button>
            </DropdownMenu.Trigger>

            <DropdownMenu.Portal>
                <DropdownMenu.Content 
                    className="z-50 min-w-[280px] bg-slate-900 border border-slate-700 rounded-md shadow-xl p-1 animate-in fade-in zoom-in-95 duration-100"
                    align="start"
                    sideOffset={5}
                >
                    <div className="px-2 py-1.5 text-[10px] font-bold uppercase tracking-wider text-slate-500 border-b border-white/5 mb-1">
                        Select Reasoner Core
                    </div>
                    
                    <div className="max-h-[300px] overflow-y-auto custom-scrollbar">
                        {models.map((model) => (
                            <DropdownMenu.Item
                                key={model.id}
                                className={`
                                    flex items-start gap-3 px-2 py-2 rounded text-xs outline-none cursor-pointer
                                    ${selectedModel === model.id ? 'bg-emerald-500/10' : 'hover:bg-white/5 focus:bg-white/5'}
                                `}
                                onSelect={() => setSelectedModel(model.id)}
                            >
                                <div className="mt-0.5">{getIcon(model.specialization)}</div>
                                <div className="flex-1">
                                    <div className="flex items-center justify-between mb-0.5">
                                        <span className={`font-medium ${selectedModel === model.id ? 'text-emerald-400' : 'text-slate-300'}`}>
                                            {model.display_name}
                                        </span>
                                        {selectedModel === model.id && <Check className="w-3 h-3 text-emerald-500" />}
                                    </div>
                                    <div className="text-[10px] text-slate-500 leading-tight">
                                        {model.specialization} · {(model.context_window / 1000).toFixed(0)}k ctx
                                    </div>
                                </div>
                            </DropdownMenu.Item>
                        ))}
                    </div>
                    
                    <div className="px-2 py-1.5 border-t border-white/5 mt-1 bg-slate-950/30 rounded-b">
                        <div className="flex items-center justify-between text-[9px] text-slate-500">
                            <span>Powered by OpenRouter</span>
                            <span className="text-emerald-500/50">● Connected</span>
                        </div>
                    </div>
                </DropdownMenu.Content>
            </DropdownMenu.Portal>
        </DropdownMenu.Root>
    );
};

export default ModelSelector;
