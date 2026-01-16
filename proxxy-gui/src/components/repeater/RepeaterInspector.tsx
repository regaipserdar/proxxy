import React, { useCallback, useMemo } from 'react';
import { Search, Terminal, ChevronRight } from 'lucide-react';
import Editor from '@monaco-editor/react';
import { ResizablePanelGroup, ResizablePanel, ResizableHandle } from '@/components/ui/resizable';
import { RepeaterTask } from '@/types';

interface RepeaterInspectorProps {
    activeTask: RepeaterTask | undefined;
    updateTask: (id: string, updates: Partial<RepeaterTask>) => void;
    reqSearch: string;
    setReqSearch: (val: string) => void;
    resSearch: string;
    setResSearch: (val: string) => void;
}

// Monaco editor custom theme
const monacoTheme = {
    base: 'vs-dark' as const,
    inherit: true,
    rules: [
        { token: '', foreground: 'D4D4D4', background: '0D0F13' },
        { token: 'keyword', foreground: '9DCDE8', fontStyle: 'bold' },
        { token: 'string', foreground: '7EC699' },
        { token: 'number', foreground: 'B5CEA8' },
        { token: 'comment', foreground: '6A9955', fontStyle: 'italic' },
        { token: 'type', foreground: '4EC9B0' },
    ],
    colors: {
        'editor.background': '#0D0F13',
        'editor.foreground': '#D4D4D4',
        'editor.lineHighlightBackground': '#ffffff08',
        'editor.selectionBackground': '#9DCDE830',
        'editorCursor.foreground': '#9DCDE8',
        'editorLineNumber.foreground': '#ffffff20',
        'editorLineNumber.activeForeground': '#9DCDE8',
        'editor.inactiveSelectionBackground': '#9DCDE815',
    },
};

const responseTheme = {
    ...monacoTheme,
    colors: {
        ...monacoTheme.colors,
        'editor.background': '#080A0E',
        'editorCursor.foreground': '#34D399',
        'editorLineNumber.activeForeground': '#34D399',
        'editor.selectionBackground': '#34D39930',
    },
};

export const RepeaterInspector: React.FC<RepeaterInspectorProps> = ({
    activeTask,
    updateTask,
    reqSearch,
    setReqSearch,
    resSearch,
    setResSearch
}) => {
    // Parse response for status code display
    const responseInfo = useMemo(() => {
        if (!activeTask?.response) return null;
        const firstLine = activeTask.response.split('\n')[0];
        const match = firstLine.match(/HTTP\/[\d.]+ (\d+)/);
        const statusCode = match ? parseInt(match[1]) : null;
        const isSuccess = statusCode && statusCode >= 200 && statusCode < 300;
        const isError = statusCode && statusCode >= 400;
        const size = Math.round(activeTask.response.length / 1024 * 10) / 10;
        return { statusCode, isSuccess, isError, size };
    }, [activeTask?.response]);

    const handleRequestChange = useCallback((value: string | undefined) => {
        if (activeTask && value !== undefined) {
            updateTask(activeTask.id, { request: value });
        }
    }, [activeTask, updateTask]);

    const handleEditorMount = useCallback((_editor: unknown, monaco: any) => {
        // Define custom themes
        monaco.editor.defineTheme('repeater-request', monacoTheme);
        monaco.editor.defineTheme('repeater-response', responseTheme);
    }, []);

    return (
        <div className="flex-1 overflow-hidden">
            <ResizablePanelGroup direction="horizontal" className="h-full">
                {/* Request Panel */}
                <ResizablePanel defaultSize={50} minSize={25}>
                    <div className="flex flex-col h-full bg-[#0D0F13]">
                        {/* Request Header */}
                        <div className="px-4 py-2.5 border-b border-white/5 flex items-center justify-between bg-gradient-to-r from-white/[0.02] to-transparent">
                            <div className="flex items-center gap-3">
                                <div className="flex items-center gap-2">
                                    <div className="w-2 h-2 rounded-full bg-[#9DCDE8] animate-pulse" />
                                    <span className="text-[11px] font-semibold uppercase tracking-wider text-[#9DCDE8]">
                                        Request
                                    </span>
                                </div>
                            </div>
                            <div className="relative group">
                                <Search size={12} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-white/20 group-focus-within:text-[#9DCDE8] transition-colors" />
                                <input
                                    type="text"
                                    placeholder="Find..."
                                    value={reqSearch}
                                    onChange={(e) => setReqSearch(e.target.value)}
                                    className="w-32 bg-black/30 border border-white/5 rounded-md px-7 py-1.5 text-[10px] text-white/60 focus:outline-none focus:border-[#9DCDE8]/40 focus:text-white focus:w-48 font-mono transition-all duration-200"
                                />
                            </div>
                        </div>

                        {/* Monaco Editor for Request */}
                        <div className="flex-1 overflow-hidden">
                            <Editor
                                height="100%"
                                defaultLanguage="http"
                                language="http"
                                value={activeTask?.request || ''}
                                onChange={handleRequestChange}
                                onMount={handleEditorMount}
                                theme="repeater-request"
                                options={{
                                    minimap: { enabled: false },
                                    fontSize: 13,
                                    fontFamily: "'JetBrains Mono', 'Fira Code', Consolas, monospace",
                                    lineHeight: 22,
                                    padding: { top: 16, bottom: 16 },
                                    scrollBeyondLastLine: false,
                                    wordWrap: 'on',
                                    automaticLayout: true,
                                    scrollbar: {
                                        vertical: 'auto',
                                        horizontal: 'auto',
                                        verticalScrollbarSize: 8,
                                        horizontalScrollbarSize: 8,
                                    },
                                    overviewRulerBorder: false,
                                    hideCursorInOverviewRuler: true,
                                    renderLineHighlight: 'line',
                                    lineNumbers: 'on',
                                    glyphMargin: false,
                                    folding: false,
                                    lineDecorationsWidth: 8,
                                    lineNumbersMinChars: 3,
                                    renderWhitespace: 'selection',
                                }}
                            />
                        </div>
                    </div>
                </ResizablePanel>

                {/* Resizable Handle */}
                <ResizableHandle className="w-1.5 bg-white/5 hover:bg-[#9DCDE8]/30 active:bg-[#9DCDE8]/50 transition-colors data-[resize-handle-state=drag]:bg-[#9DCDE8]/50" />

                {/* Response Panel */}
                <ResizablePanel defaultSize={50} minSize={25}>
                    <div className="flex flex-col h-full bg-[#080A0E]">
                        {/* Response Header */}
                        <div className="px-4 py-2.5 border-b border-white/5 flex items-center justify-between bg-gradient-to-r from-white/[0.02] to-transparent">
                            <div className="flex items-center gap-3">
                                <div className="flex items-center gap-2">
                                    <ChevronRight size={14} className="text-emerald-400" />
                                    <span className="text-[11px] font-semibold uppercase tracking-wider text-emerald-400">
                                        Response
                                    </span>
                                </div>

                                {responseInfo && (
                                    <div className="flex gap-2 items-center text-[10px] font-mono">
                                        <span className={`px-2 py-0.5 rounded-md border ${responseInfo.isSuccess
                                                ? 'text-emerald-400 bg-emerald-400/10 border-emerald-400/20'
                                                : responseInfo.isError
                                                    ? 'text-red-400 bg-red-400/10 border-red-400/20'
                                                    : 'text-amber-400 bg-amber-400/10 border-amber-400/20'
                                            }`}>
                                            {responseInfo.statusCode}
                                        </span>
                                        <span className="text-white/30">{responseInfo.size} KB</span>
                                    </div>
                                )}
                            </div>
                            <div className="relative group">
                                <Search size={12} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-white/20 group-focus-within:text-emerald-400 transition-colors" />
                                <input
                                    type="text"
                                    placeholder="Find..."
                                    value={resSearch}
                                    onChange={(e) => setResSearch(e.target.value)}
                                    className="w-32 bg-black/30 border border-white/5 rounded-md px-7 py-1.5 text-[10px] text-white/60 focus:outline-none focus:border-emerald-500/40 focus:text-white focus:w-48 font-mono transition-all duration-200"
                                />
                            </div>
                        </div>

                        {/* Response Content */}
                        {activeTask?.response ? (
                            <div className="flex-1 overflow-hidden">
                                <Editor
                                    height="100%"
                                    defaultLanguage="http"
                                    language="http"
                                    value={activeTask.response}
                                    onMount={handleEditorMount}
                                    theme="repeater-response"
                                    options={{
                                        readOnly: true,
                                        minimap: { enabled: false },
                                        fontSize: 13,
                                        fontFamily: "'JetBrains Mono', 'Fira Code', Consolas, monospace",
                                        lineHeight: 22,
                                        padding: { top: 16, bottom: 16 },
                                        scrollBeyondLastLine: false,
                                        wordWrap: 'on',
                                        automaticLayout: true,
                                        scrollbar: {
                                            vertical: 'auto',
                                            horizontal: 'auto',
                                            verticalScrollbarSize: 8,
                                            horizontalScrollbarSize: 8,
                                        },
                                        overviewRulerBorder: false,
                                        hideCursorInOverviewRuler: true,
                                        renderLineHighlight: 'none',
                                        lineNumbers: 'on',
                                        glyphMargin: false,
                                        folding: false,
                                        lineDecorationsWidth: 8,
                                        lineNumbersMinChars: 3,
                                    }}
                                />
                            </div>
                        ) : (
                            <div className="flex-1 flex flex-col items-center justify-center opacity-30 gap-4 select-none">
                                <div className="w-20 h-20 rounded-2xl bg-gradient-to-br from-white/10 to-white/5 flex items-center justify-center border border-white/5">
                                    <Terminal size={36} className="text-white/40" />
                                </div>
                                <div className="text-center">
                                    <p className="text-xs font-bold uppercase tracking-widest mb-1 text-white/60">
                                        Waiting for Response
                                    </p>
                                    <p className="text-[10px] text-white/40">
                                        Select an agent and click Send
                                    </p>
                                </div>
                            </div>
                        )}
                    </div>
                </ResizablePanel>
            </ResizablePanelGroup>
        </div>
    );
};
