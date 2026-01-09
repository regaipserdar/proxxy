import React from 'react';
import { Terminal, RefreshCw, AlertCircle, Copy, Check } from 'lucide-react';

interface Props {
    onRetry: () => void;
}

export function OrchestratorConnectionError({ onRetry }: Props) {
    const [copied, setCopied] = React.useState(false);
    const command = "cargo run -p orchestrator -- --grpc-port 50051 --http-port 9090";

    const handleCopy = () => {
        navigator.clipboard.writeText(command);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };

    return (
        <div className="min-h-screen bg-slate-950 flex items-center justify-center p-6 text-slate-200 font-sans">
            <div className="max-w-2xl w-full bg-slate-900/50 border border-red-900/50 rounded-xl p-8 backdrop-blur-sm shadow-2xl">
                <div className="flex items-center space-x-4 mb-6 text-red-400">
                    <div className="p-3 bg-red-950/50 rounded-full">
                        <AlertCircle className="w-8 h-8" />
                    </div>
                    <div>
                        <h1 className="text-2xl font-bold text-slate-100">Connection Failed</h1>
                        <p className="text-red-400/80">Cannot connect to Proxxy Orchestrator</p>
                    </div>
                </div>

                <div className="space-y-6">
                    <p className="text-slate-400">
                        The backend service is not running or is unreachable.
                        Please ensure the orchestrator is running in your terminal.
                    </p>

                    <div className="bg-slate-950 border border-slate-800 rounded-lg overflow-hidden group">
                        <div className="flex items-center justify-between px-4 py-2 bg-slate-900 border-b border-slate-800">
                            <div className="flex items-center space-x-2 text-slate-500 text-xs">
                                <Terminal className="w-3 h-3" />
                                <span>Terminal</span>
                            </div>
                            <button
                                onClick={handleCopy}
                                className="text-xs flex items-center space-x-1 text-slate-500 hover:text-emerald-400 transition-colors"
                            >
                                {copied ? <Check className="w-3 h-3" /> : <Copy className="w-3 h-3" />}
                                <span>{copied ? 'Copied' : 'Copy'}</span>
                            </button>
                        </div>
                        <div className="p-4 font-mono text-sm text-emerald-400 overflow-x-auto whitespace-nowrap selection:bg-emerald-900/50">
                            {command}
                        </div>
                    </div>

                    <div className="pt-4 flex justify-end">
                        <button
                            onClick={onRetry}
                            className="flex items-center space-x-2 px-6 py-2.5 bg-slate-800 hover:bg-slate-700 text-white rounded-lg font-medium transition-all hover:shadow-lg hover:shadow-emerald-900/20 active:scale-95 border border-slate-700 hover:border-slate-600"
                        >
                            <RefreshCw className="w-4 h-4" />
                            <span>Try Again</span>
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
}
