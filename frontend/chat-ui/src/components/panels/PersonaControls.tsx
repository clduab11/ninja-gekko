import { FormEvent, useState } from 'react';
import { Settings2 } from 'lucide-react';

import { PersonaSettings } from '../../types';

interface Props {
  persona: PersonaSettings;
  onSave: (persona: PersonaSettings) => Promise<void> | void;
  isLoading?: boolean;
}

const toneOptions: PersonaSettings['tone'][] = ['concise', 'balanced', 'dramatic'];
const styleOptions: PersonaSettings['style'][] = ['analytical', 'witty', 'direct'];
const moodOptions: PersonaSettings['mood'][] = ['direct', 'witty', 'calm'];

const PersonaControls = ({ persona, onSave, isLoading }: Props) => {
  const [draft, setDraft] = useState<PersonaSettings>(persona);

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    await onSave(draft);
  };

  return (
    <section 
      className="rounded-xl border border-border/60 bg-panel/80 p-5 text-sm"
      data-testid="persona-controls"
      role="region"
      aria-label="Gordon persona configuration"
    >
      <header className="mb-3 flex items-center gap-2 text-xs uppercase tracking-[0.35em] text-white/40">
        <Settings2 className="h-4 w-4 text-accent" aria-hidden="true" /> Persona Tuning
      </header>
      <form 
        onSubmit={handleSubmit} 
        className="space-y-4"
        data-testid="persona-form"
        role="form"
        aria-label="Configure Gordon persona settings"
      >
        <label className="grid gap-1">
          <span className="text-xs text-white/50" id="label-persona-tone">Tone</span>
          <select
            value={draft.tone}
            onChange={(event) => setDraft({ ...draft, tone: event.target.value as PersonaSettings['tone'] })}
            className="rounded-lg border border-border/60 bg-panel px-3 py-2 text-white"
            data-testid="select-persona-tone"
            aria-labelledby="label-persona-tone"
            aria-label="Select persona tone"
          >
            {toneOptions.map((option) => (
              <option key={option} value={option}>
                {option}
              </option>
            ))}
          </select>
        </label>

        <label className="grid gap-1">
          <span className="text-xs text-white/50" id="label-persona-style">Style</span>
          <select
            value={draft.style}
            onChange={(event) => setDraft({ ...draft, style: event.target.value as PersonaSettings['style'] })}
            className="rounded-lg border border-border/60 bg-panel px-3 py-2 text-white"
            data-testid="select-persona-style"
            aria-labelledby="label-persona-style"
            aria-label="Select persona style"
          >
            {styleOptions.map((option) => (
              <option key={option} value={option}>
                {option}
              </option>
            ))}
          </select>
        </label>

        <label className="grid gap-1">
          <span className="text-xs text-white/50" id="label-persona-mood">Mood</span>
          <select
            value={draft.mood}
            onChange={(event) => setDraft({ ...draft, mood: event.target.value as PersonaSettings['mood'] })}
            className="rounded-lg border border-border/60 bg-panel px-3 py-2 text-white"
            data-testid="select-persona-mood"
            aria-labelledby="label-persona-mood"
            aria-label="Select persona mood"
          >
            {moodOptions.map((option) => (
              <option key={option} value={option}>
                {option}
              </option>
            ))}
          </select>
        </label>

        <button
          type="submit"
          className="w-full rounded-lg bg-accent py-2 text-sm font-semibold text-black disabled:cursor-not-allowed"
          disabled={isLoading}
          data-testid="btn-update-persona"
          role="button"
          aria-label={isLoading ? 'Syncing persona settings...' : 'Update Gordon persona settings'}
          aria-disabled={isLoading}
        >
          {isLoading ? 'Syncing persona…' : 'Update persona'}
        </button>
      </form>
    </section>
  );
};

export default PersonaControls;
