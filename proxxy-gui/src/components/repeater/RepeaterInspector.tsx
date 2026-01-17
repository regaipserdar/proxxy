import React, { useCallback, useMemo, useState, useRef } from 'react';
import { Terminal, ChevronRight, ExternalLink, FileText, Code, Eye, ChevronDown } from 'lucide-react';
import Editor from '@monaco-editor/react';
import { ResizablePanelGroup, ResizablePanel, ResizableHandle } from '@/components/ui/resizable';
import { RepeaterTask } from '@/store/repeaterStore';
import { useMutation } from '@apollo/client';
import { OPEN_RESPONSE_IN_BROWSER } from '@/graphql/operations';

interface RepeaterInspectorProps {
    activeTask: RepeaterTask | undefined;
    updateTask: (id: string, updates: Partial<RepeaterTask>) => void;
}

// Available request languages for syntax highlighting
const REQUEST_LANGUAGES = [
    { value: 'http', label: 'HTTP' },
    { value: 'json', label: 'JSON' },
    { value: 'xml', label: 'XML' },
    { value: 'html', label: 'HTML' },
    { value: 'javascript', label: 'JavaScript' },
    { value: 'plaintext', label: 'Plain Text' },
];

// HTTP methods for context menu
const HTTP_METHODS = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS', 'TRACE'];

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

type ResponseViewMode = 'raw' | 'headers' | 'body' | 'render';

interface ContextMenuState {
    x: number;
    y: number;
    visible: boolean;
}

// Parse response into parts
const parseResponse = (raw: string) => {
    if (!raw) return { headers: '', body: '', statusLine: '' };

    const normalized = raw.replace(/\r\n/g, '\n');
    const parts = normalized.split('\n\n');
    const headerPart = parts[0] || '';
    const bodyPart = parts.slice(1).join('\n\n');

    const lines = headerPart.split('\n');
    const statusLine = lines[0] || '';
    const headers = lines.slice(1).join('\n');

    return { statusLine, headers, body: bodyPart };
};

// Detect language from Content-Type
const detectLanguage = (headers: string): string => {
    const contentTypeLine = headers.split('\n').find(line =>
        line.toLowerCase().startsWith('content-type:')
    );
    if (!contentTypeLine) return 'plaintext';

    const contentType = contentTypeLine.toLowerCase();
    if (contentType.includes('json')) return 'json';
    if (contentType.includes('xml')) return 'xml';
    if (contentType.includes('html')) return 'html';
    if (contentType.includes('javascript')) return 'javascript';
    if (contentType.includes('css')) return 'css';
    return 'plaintext';
};

export const RepeaterInspector: React.FC<RepeaterInspectorProps> = ({
    activeTask,
    updateTask
}) => {
    const [responseViewMode, setResponseViewMode] = useState<ResponseViewMode>('raw');
    const [requestLanguage, setRequestLanguage] = useState<string>('http');
    const [showLangDropdown, setShowLangDropdown] = useState(false);
    const [methodMenu, setMethodMenu] = useState<ContextMenuState>({ x: 0, y: 0, visible: false });
    const [openInBrowserMutation] = useMutation(OPEN_RESPONSE_IN_BROWSER);
    const editorContainerRef = useRef<HTMLDivElement>(null);

    // Parse response for status code display and view modes
    const responseInfo = useMemo(() => {
        if (!activeTask?.response) return null;
        const parsed = parseResponse(activeTask.response);
        const match = parsed.statusLine.match(/HTTP\/[\d.]+ (\d+)/);
        const statusCode = match ? parseInt(match[1]) : null;
        const isSuccess = statusCode && statusCode >= 200 && statusCode < 300;
        const isError = statusCode && statusCode >= 400;
        const size = Math.round(activeTask.response.length / 1024 * 10) / 10;
        const language = detectLanguage(parsed.headers);
        return { statusCode, isSuccess, isError, size, ...parsed, language };
    }, [activeTask?.response]);

    const handleRequestChange = useCallback((value: string | undefined) => {
        if (activeTask && value !== undefined) {
            updateTask(activeTask.id, { request: value });
        }
    }, [activeTask, updateTask]);

    const handleEditorMount = useCallback((_editor: unknown, monaco: any) => {
        // Register custom HTTP language with colorful syntax
        monaco.languages.register({ id: 'http' });
        monaco.languages.setMonarchTokensProvider('http', {
            tokenizer: {
                root: [
                    // HTTP Methods (bold red/orange)
                    [/^(GET|POST|PUT|PATCH|DELETE|HEAD|OPTIONS|TRACE|CONNECT)\b/, 'http-method'],
                    // HTTP Version
                    [/HTTP\/[\d.]+/, 'http-version'],
                    // Status codes
                    [/\b[1-5]\d{2}\b/, 'http-status'],
                    // URLs and paths
                    [/https?:\/\/[^\s]+/, 'http-url'],
                    [/^\/[^\s]*/, 'http-path'],
                    // Headers (key: value)
                    [/^([A-Za-z-]+)(:)/, ['http-header-key', 'http-colon']],
                    // JSON in body
                    [/"[^"]*"/, 'string'],
                    [/\b\d+\b/, 'number'],
                    [/[{}[\]]/, 'delimiter.bracket'],
                    // Comments
                    [/#.*$/, 'comment'],
                ]
            }
        });

        // Define custom themes with HTTP colors
        monaco.editor.defineTheme('repeater-request', {
            ...monacoTheme,
            rules: [
                ...monacoTheme.rules,
                { token: 'http-method', foreground: 'FF79C6', fontStyle: 'bold' }, // Pink/magenta for methods
                { token: 'http-version', foreground: '6272A4' }, // Dim for HTTP version
                { token: 'http-status', foreground: '50FA7B', fontStyle: 'bold' }, // Green for status
                { token: 'http-url', foreground: '8BE9FD' }, // Cyan for URLs
                { token: 'http-path', foreground: 'BD93F9' }, // Purple for paths
                { token: 'http-header-key', foreground: 'FFB86C' }, // Orange for header names
                { token: 'http-colon', foreground: 'F8F8F2' }, // White for colon
                { token: 'string', foreground: 'F1FA8C' }, // Yellow for strings
                { token: 'number', foreground: 'BD93F9' }, // Purple for numbers
                { token: 'comment', foreground: '6272A4', fontStyle: 'italic' },
            ],
        });

        monaco.editor.defineTheme('repeater-response', {
            ...responseTheme,
            rules: [
                ...responseTheme.rules,
                { token: 'http-method', foreground: 'FF79C6', fontStyle: 'bold' },
                { token: 'http-version', foreground: '6272A4' },
                { token: 'http-status', foreground: '50FA7B', fontStyle: 'bold' },
                { token: 'http-url', foreground: '8BE9FD' },
                { token: 'http-path', foreground: 'BD93F9' },
                { token: 'http-header-key', foreground: 'FFB86C' },
                { token: 'http-colon', foreground: 'F8F8F2' },
                { token: 'string', foreground: 'F1FA8C' },
                { token: 'number', foreground: 'BD93F9' },
                { token: 'comment', foreground: '6272A4', fontStyle: 'italic' },
            ],
        });
    }, []);

    // Change HTTP method in request
    const handleChangeMethod = useCallback((newMethod: string) => {
        if (!activeTask?.request) return;

        const lines = activeTask.request.split('\n');
        if (lines.length === 0) return;

        // Parse first line: "GET /path HTTP/1.1" -> replace method
        const firstLine = lines[0];
        const parts = firstLine.split(' ');
        if (parts.length >= 2) {
            parts[0] = newMethod;
            lines[0] = parts.join(' ');
            updateTask(activeTask.id, { request: lines.join('\n') });
        }

        setMethodMenu({ x: 0, y: 0, visible: false });
    }, [activeTask, updateTask]);

    // Handle right-click on editor
    const handleEditorContextMenu = useCallback((e: React.MouseEvent) => {
        e.preventDefault();
        setMethodMenu({ x: e.clientX, y: e.clientY, visible: true });
    }, []);

    // Close method menu on click outside
    const handleCloseMethodMenu = useCallback(() => {
        setMethodMenu({ x: 0, y: 0, visible: false });
    }, []);

    // Open response in orchestrator's managed browser
    const handleOpenInBrowser = useCallback(async () => {
        if (!responseInfo?.body || !activeTask?.request) return;

        try {
            const contentType = responseInfo.language === 'html' ? 'text/html' :
                responseInfo.language === 'json' ? 'application/json' :
                    'text/plain';

            // Extract base URL from request for proper relative URL resolution
            let baseUrl = undefined;
            const requestLines = activeTask.request.split('\n');
            if (requestLines.length > 0) {
                // Try to extract URL from first line (e.g., "GET https://example.com/path HTTP/1.1")
                const firstLine = requestLines[0];
                const urlMatch = firstLine.match(/\s+(https?:\/\/[^\s]+)/);
                if (urlMatch) {
                    try {
                        const url = new URL(urlMatch[1]);
                        // Use origin as base (e.g., https://example.com)
                        baseUrl = url.origin;
                    } catch {
                        // Invalid URL, skip
                    }
                }
            }

            // Use GraphQL mutation to open in orchestrator's browser
            await openInBrowserMutation({
                variables: {
                    content: responseInfo.body,
                    contentType: contentType,
                    baseUrl: baseUrl
                }
            });
        } catch (error) {
            console.error('Failed to open in browser:', error);
        }
    }, [responseInfo, openInBrowserMutation, activeTask]);

    // Get content for current view mode
    const getResponseContent = () => {
        if (!responseInfo) return '';
        switch (responseViewMode) {
            case 'headers':
                return `${responseInfo.statusLine}\n${responseInfo.headers}`;
            case 'body':
                return responseInfo.body;
            case 'render':
                return responseInfo.body;
            case 'raw':
            default:
                return activeTask?.response || '';
        }
    };

    const getResponseLanguage = () => {
        if (responseViewMode === 'headers' || responseViewMode === 'raw') return 'http';
        return responseInfo?.language || 'plaintext';
    };

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
                                {/* Language Selector */}
                                <div className="relative">
                                    <button
                                        onClick={() => setShowLangDropdown(!showLangDropdown)}
                                        className="flex items-center gap-1 px-2 py-1 text-[10px] text-white/50 hover:text-white/80 bg-white/5 hover:bg-white/10 rounded transition-colors"
                                    >
                                        {REQUEST_LANGUAGES.find(l => l.value === requestLanguage)?.label || 'HTTP'}
                                        <ChevronDown size={10} />
                                    </button>
                                    {showLangDropdown && (
                                        <div className="absolute top-full left-0 mt-1 bg-[#1a1d24] border border-white/10 rounded-md shadow-xl z-50 min-w-[100px] py-1">
                                            {REQUEST_LANGUAGES.map(lang => (
                                                <button
                                                    key={lang.value}
                                                    onClick={() => {
                                                        setRequestLanguage(lang.value);
                                                        setShowLangDropdown(false);
                                                    }}
                                                    className={`w-full px-3 py-1.5 text-left text-[10px] hover:bg-white/10 transition-colors ${requestLanguage === lang.value ? 'text-[#9DCDE8]' : 'text-white/60'
                                                        }`}
                                                >
                                                    {lang.label}
                                                </button>
                                            ))}
                                        </div>
                                    )}
                                </div>
                            </div>
                            {/* Keyboard shortcut hint */}
                            <div className="text-[9px] text-white/30 font-mono">
                                Ctrl+F Search • Ctrl+H Replace
                            </div>
                        </div>

                        {/* Monaco Editor for Request */}
                        <div
                            ref={editorContainerRef}
                            className="flex-1 overflow-hidden"
                            onContextMenu={handleEditorContextMenu}
                        >
                            <Editor
                                height="100%"
                                defaultLanguage="http"
                                language={requestLanguage}
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
                                    contextmenu: false, // Disable Monaco's context menu for custom menu
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

                            <div className="flex items-center gap-2">
                                {/* View Mode Tabs */}
                                {activeTask?.response && (
                                    <div className="flex items-center gap-0.5 bg-black/30 rounded-md p-0.5 border border-white/5">
                                        <button
                                            onClick={() => setResponseViewMode('raw')}
                                            className={`px-2 py-1 rounded text-[9px] font-bold uppercase transition-colors ${responseViewMode === 'raw'
                                                ? 'bg-emerald-400/20 text-emerald-400'
                                                : 'text-white/40 hover:text-white/60'
                                                }`}
                                            title="Raw HTTP response"
                                        >
                                            <Code size={10} />
                                        </button>
                                        <button
                                            onClick={() => setResponseViewMode('headers')}
                                            className={`px-2 py-1 rounded text-[9px] font-bold uppercase transition-colors ${responseViewMode === 'headers'
                                                ? 'bg-emerald-400/20 text-emerald-400'
                                                : 'text-white/40 hover:text-white/60'
                                                }`}
                                            title="Headers only"
                                        >
                                            <FileText size={10} />
                                        </button>
                                        <button
                                            onClick={() => setResponseViewMode('body')}
                                            className={`px-2 py-1 rounded text-[9px] font-bold uppercase transition-colors ${responseViewMode === 'body'
                                                ? 'bg-emerald-400/20 text-emerald-400'
                                                : 'text-white/40 hover:text-white/60'
                                                }`}
                                            title="Body only"
                                        >
                                            <Eye size={10} />
                                        </button>
                                    </div>
                                )}

                                {/* Open in Browser Button */}
                                {activeTask?.response && responseInfo?.body && (
                                    <button
                                        onClick={handleOpenInBrowser}
                                        className="flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-emerald-400/10 hover:bg-emerald-400/20 border border-emerald-400/20 text-emerald-400 text-[10px] font-bold transition-colors"
                                        title="Open response in browser"
                                    >
                                        <ExternalLink size={10} />
                                        Browser
                                    </button>
                                )}

                                {/* Keyboard shortcut hint */}
                                <div className="text-[9px] text-white/30 font-mono">
                                    Ctrl+F Search
                                </div>
                            </div>
                        </div>

                        {/* Response Content */}
                        {activeTask?.response ? (
                            <div className="flex-1 overflow-hidden">
                                <Editor
                                    height="100%"
                                    defaultLanguage="http"
                                    language={getResponseLanguage()}
                                    value={getResponseContent()}
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

            {/* HTTP Method Context Menu */}
            {methodMenu.visible && (
                <div
                    className="fixed inset-0 z-50"
                    onClick={handleCloseMethodMenu}
                >
                    <div
                        className="absolute bg-[#1a1d24] border border-white/10 rounded-md shadow-2xl py-1 min-w-[120px]"
                        style={{ left: methodMenu.x, top: methodMenu.y }}
                        onClick={(e) => e.stopPropagation()}
                    >
                        <div className="px-3 py-1.5 text-[9px] text-white/30 uppercase tracking-wider border-b border-white/5">
                            Change Method
                        </div>
                        {HTTP_METHODS.map(method => (
                            <button
                                key={method}
                                onClick={() => handleChangeMethod(method)}
                                className="w-full px-3 py-1.5 text-left text-[11px] text-white/70 hover:bg-white/10 hover:text-white transition-colors font-mono"
                            >
                                {method}
                            </button>
                        ))}
                    </div>
                </div>
            )}
        </div>
    );
};

