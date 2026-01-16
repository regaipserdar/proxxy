import { useState } from 'react';
import { useQuery, useMutation } from '@apollo/client';
import {
  Plus, Trash2, Target,
  Zap,
  LayoutGrid,
  Shield,
  ShieldAlert,
  ToggleLeft,
  ToggleRight,
  Info
} from 'lucide-react';
import { ScopeRule } from '@/types';
import {
  GET_SCOPE_RULES,
  ADD_SCOPE_RULE,
  DELETE_SCOPE_RULE,
  TOGGLE_SCOPE_RULE
} from '@/graphql/operations';
import { toast } from 'sonner';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Card } from '@/components/ui/card';

export const ScopeManager = () => {
  const { data, loading, error, refetch } = useQuery(GET_SCOPE_RULES);
  const [addScopeRule] = useMutation(ADD_SCOPE_RULE);
  const [deleteScopeRuleMutation] = useMutation(DELETE_SCOPE_RULE);
  const [toggleScopeRuleMutation] = useMutation(TOGGLE_SCOPE_RULE);

  const [pattern, setPattern] = useState('');
  const [ruleType, setRuleType] = useState('Include');
  const [isRegex, setIsRegex] = useState(false);
  const [isAdding, setIsAdding] = useState(false);

  const rules: ScopeRule[] = data?.scopeRules || [];

  const handleAddRule = async () => {
    if (!pattern.trim()) {
      toast.error('Pattern cannot be empty');
      return;
    }

    try {
      setIsAdding(true);
      await addScopeRule({
        variables: {
          ruleType,
          pattern,
          isRegex
        }
      });
      toast.success('Rule added successfully');
      setPattern('');
      refetch();
    } catch (err: any) {
      toast.error(`Failed to add rule: ${err.message}`);
    } finally {
      setIsAdding(false);
    }
  };

  const handleDeleteRule = async (id: string) => {
    try {
      await deleteScopeRuleMutation({
        variables: { id }
      });
      toast.success('Rule deleted');
      refetch();
    } catch (err: any) {
      toast.error(`Error deleting rule: ${err.message}`);
    }
  };

  const handleToggleRule = async (id: string, enabled: boolean) => {
    try {
      await toggleScopeRuleMutation({
        variables: { id, enabled }
      });
      refetch();
    } catch (err: any) {
      toast.error(`Error updating rule: ${err.message}`);
    }
  };

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-white/40">
        <ShieldAlert size={48} className="mb-4 text-red-500/50" />
        <p className="text-lg">Failed to load scope rules</p>
        <p className="text-sm">{error.message}</p>
        <Button variant="outline" className="mt-4" onClick={() => refetch()}>Retry</Button>
      </div>
    );
  }

  return (
    <div className="flex h-full bg-[#0A0E14] overflow-hidden select-none">

      {/* Rules Manager Area */}
      <div className="flex-1 flex flex-col min-w-0 font-sans">

        {/* Header */}
        <div className="h-16 border-b border-white/5 flex items-center justify-between px-8 bg-[#0D0F13]">
          <div className="flex items-center gap-4">
            <div className="w-10 h-10 rounded-xl bg-orange-500/10 flex items-center justify-center border border-orange-500/20 shadow-lg shadow-orange-500/5">
              <Target size={20} className="text-orange-400" />
            </div>
            <div>
              <h2 className="text-base font-bold text-white tracking-tight">Target Scope</h2>
              <p className="text-[10px] text-white/30 uppercase tracking-widest font-medium">Capture Rules & Filtering</p>
            </div>
          </div>

          <div className="flex items-center gap-4">
            <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-blue-500/5 border border-blue-500/10">
              <div className="w-1.5 h-1.5 rounded-full bg-blue-500 animate-pulse" />
              <span className="text-[10px] font-bold text-blue-400 uppercase tracking-widest">Realtime Sync</span>
            </div>
          </div>
        </div>

        <div className="flex-1 overflow-y-auto w-full max-w-6xl mx-auto p-8 space-y-8">

          {/* Quick Info */}
          <Card className="bg-gradient-to-br from-blue-500/5 to-purple-500/5 border-white/10 p-6 relative overflow-hidden group">
            <div className="absolute top-0 right-0 w-32 h-32 bg-blue-500/10 blur-[60px] rounded-full -mr-16 -mt-16 transition-all group-hover:bg-blue-500/20" />
            <div className="flex items-start gap-5 relative z-10">
              <div className="p-3 bg-blue-500/20 rounded-2xl border border-blue-500/20">
                <Info size={24} className="text-blue-400" />
              </div>
              <div className="space-y-2">
                <h3 className="text-sm font-bold text-white">How filtering works</h3>
                <p className="text-xs text-white/50 leading-relaxed max-w-2xl">
                  By default, Orchestrator captures all traffic. If you add <span className="text-emerald-400 font-bold underline decoration-emerald-500/30 underline-offset-4">Include</span> rules, only traffic matching those rules will be recorded. <span className="text-red-400 font-bold underline decoration-red-500/30 underline-offset-4">Exclude</span> rules always take precedence and will block matching traffic even if it matches an include rule.
                  <br />
                  <span className="mt-1 block text-white/30 italic italic">Tunnels (CONNECT :443) are automatically ignored to keep your database clean.</span>
                </p>
              </div>
            </div>
          </Card>

          {/* Add Rule Form */}
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
            <Card className="p-6 bg-[#0D0F13] border-white/5 space-y-6">
              <div className="flex items-center justify-between">
                <h3 className="text-xs font-bold text-white/40 uppercase tracking-[0.2em] flex items-center gap-2">
                  <Plus size={14} className="text-[#9DCDE8]" />
                  New Rule
                </h3>
                <div className="flex bg-black/40 p-1 rounded-lg border border-white/5">
                  <button
                    onClick={() => setRuleType('Include')}
                    className={`px-3 py-1 rounded-md text-[10px] font-bold uppercase transition-all ${ruleType === 'Include' ? 'bg-emerald-500 text-black shadow-lg shadow-emerald-500/20' : 'text-white/20 hover:text-white/40'}`}
                  >Include</button>
                  <button
                    onClick={() => setRuleType('Exclude')}
                    className={`px-3 py-1 rounded-md text-[10px] font-bold uppercase transition-all ${ruleType === 'Exclude' ? 'bg-red-500 text-black shadow-lg shadow-red-500/20' : 'text-white/20 hover:text-white/40'}`}
                  >Exclude</button>
                </div>
              </div>

              <div className="space-y-4">
                <div className="space-y-2">
                  <label className="text-[10px] font-bold text-white/20 uppercase tracking-widest ml-1">Pattern (Host or Path)</label>
                  <Input
                    value={pattern}
                    onChange={(e) => setPattern(e.target.value)}
                    placeholder={isRegex ? "^api\\..*\\.com$" : "*.example.com"}
                    className="bg-black/40 border-white/10 rounded-xl font-mono text-xs text-[#9DCDE8] placeholder:text-white/10"
                  />
                </div>

                <div className="flex items-center justify-between px-1">
                  <div className="flex items-center gap-2">
                    <span className="text-[10px] font-bold text-white/40 uppercase tracking-widest">Regex Mode</span>
                    <button
                      onClick={() => setIsRegex(!isRegex)}
                      className="text-[#9DCDE8]"
                    >
                      {isRegex ? <ToggleRight size={24} /> : <ToggleLeft size={24} className="opacity-20 translate-x-1" />}
                    </button>
                  </div>
                </div>

                <Button
                  disabled={isAdding || !pattern.trim()}
                  onClick={handleAddRule}
                  className="w-full bg-white text-black hover:bg-[#9DCDE8] font-bold rounded-xl h-10 transition-all shadow-xl shadow-black/20 uppercase tracking-widest text-[11px]"
                >
                  {isAdding ? 'Adding...' : 'Add to Scope'}
                </Button>
              </div>
            </Card>

            {/* Rules List */}
            <div className="lg:col-span-2 space-y-4">
              <div className="flex items-center justify-between px-2">
                <h3 className="text-xs font-bold text-white/40 uppercase tracking-[0.2em] flex items-center gap-2">
                  <LayoutGrid size={14} />
                  Active Rules
                </h3>
                <Badge variant="outline" className="text-[10px] border-white/10 text-white/30 font-bold px-2 py-0 border-none bg-white/5">
                  {rules.length} Rules Defined
                </Badge>
              </div>

              {loading ? (
                <div className="space-y-3">
                  {[1, 2, 3].map(i => (
                    <div key={i} className="h-16 bg-white/[0.02] border border-white/5 rounded-2xl animate-pulse" />
                  ))}
                </div>
              ) : rules.length === 0 ? (
                <div className="flex flex-col items-center justify-center p-12 border border-dashed border-white/10 rounded-3xl bg-white/[0.01]">
                  <div className="w-12 h-12 rounded-full bg-white/5 flex items-center justify-center mb-4">
                    <Shield size={24} className="text-white/20" />
                  </div>
                  <p className="text-xs text-white/30 font-medium">No scope rules defined. Recording everything.</p>
                </div>
              ) : (
                <div className="grid grid-cols-1 gap-3">
                  {rules.map(rule => (
                    <div
                      key={rule.id}
                      className={`flex items-center gap-4 px-5 py-3 rounded-2xl border transition-all group ${rule.enabled ? 'bg-white/[0.02] border-white/5 hover:border-white/10' : 'bg-transparent border-white/5 grayscale opacity-30 shadow-none'}`}
                    >
                      <Badge className={`uppercase text-[9px] font-bold tracking-widest rounded-lg px-2 h-6 flex items-center justify-center border-none ${rule.ruleType === 'Include' ? 'bg-emerald-500/10 text-emerald-400' : 'bg-red-500/10 text-red-400'}`}>
                        {rule.ruleType}
                      </Badge>

                      <div className="flex-1 flex items-center gap-2 font-mono text-[11px] min-w-0">
                        {rule.isRegex ? (
                          <div className="flex items-center gap-1.5 px-2 py-0.5 rounded-md bg-purple-500/10 border border-purple-500/20">
                            <span className="text-[8px] font-bold text-purple-400 uppercase tracking-tighter">Regex</span>
                          </div>
                        ) : (
                          <div className="flex items-center gap-1.5 px-2 py-0.5 rounded-md bg-blue-500/10 border border-blue-500/20">
                            <span className="text-[8px] font-bold text-blue-400 uppercase tracking-tighter">Glob</span>
                          </div>
                        )}
                        <span className="text-white/80 font-bold truncate">{rule.pattern}</span>
                      </div>

                      <div className="flex items-center gap-4">
                        <button
                          onClick={() => handleToggleRule(rule.id, !rule.enabled)}
                          className={`transition-all ${rule.enabled ? 'text-emerald-500' : 'text-white/20 hover:text-white/40'}`}
                        >
                          {rule.enabled ? <ToggleRight size={24} /> : <ToggleLeft size={24} />}
                        </button>

                        <div className="w-px h-4 bg-white/5" />

                        <button
                          onClick={() => handleDeleteRule(rule.id)}
                          className="p-1.5 rounded-lg text-white/20 hover:text-red-400 hover:bg-red-400/10 transition-all opacity-0 group-hover:opacity-100"
                        >
                          <Trash2 size={14} />
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>

          {/* Preset Rules / Templates */}
          <div className="space-y-4 pt-4 border-t border-white/5">
            <div className="flex items-center justify-between px-2">
              <h3 className="text-xs font-bold text-white/40 uppercase tracking-[0.2em] flex items-center gap-2">
                <Zap size={14} className="text-yellow-500/50" />
                Common Exclusions
              </h3>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {[
                { name: 'Google Analytics', pattern: '*google-analytics.com' },
                { name: 'Mixpanel Tracking', pattern: 'api.mixpanel.com' },
                { name: 'Sentry Errors', pattern: 'o*.ingest.sentry.io' },
              ].map(preset => (
                <button
                  key={preset.name}
                  className="flex flex-col items-start gap-1 p-4 rounded-2xl bg-white/[0.02] border border-white/5 hover:bg-white/[0.04] hover:border-white/10 transition-all text-left group"
                  onClick={() => {
                    setRuleType('Exclude');
                    setPattern(preset.pattern);
                    setIsRegex(false);
                    toast(`Applied preset: ${preset.name}`);
                  }}
                >
                  <span className="text-[11px] font-bold text-white group-hover:text-[#9DCDE8] transition-colors">{preset.name}</span>
                  <span className="text-[10px] text-white/30 font-mono italic">{preset.pattern}</span>
                </button>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
