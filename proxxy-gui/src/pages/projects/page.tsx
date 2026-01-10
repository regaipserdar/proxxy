import React, { useState } from 'react';
import { useQuery, useMutation } from '@apollo/client';
import { Folder, Plus, Database, HardDrive, Clock, ArrowRight, Loader2, Search, ShieldCheck, AlertCircle, Trash2, Download, Upload } from 'lucide-react';
import { GET_PROJECTS, CREATE_PROJECT, LOAD_PROJECT, DELETE_PROJECT, EXPORT_PROJECT, IMPORT_PROJECT } from '@/graphql/operations';
import { useNavigate } from 'react-router-dom';
import { OrchestratorConnectionError } from '@/components/OrchestratorConnectionError';
import { open, save } from '@tauri-apps/plugin-dialog';

export function ProjectLauncher() {
    const navigate = useNavigate();
    const [newProjectName, setNewProjectName] = useState('');
    const [searchTerm, setSearchTerm] = useState(''); // Arama için state
    const [error, setError] = useState<string | null>(null);
    const [shouldPoll, setShouldPoll] = useState(true);

    // 1. Query Hook
    const { data, loading, refetch, error: connectionError } = useQuery(GET_PROJECTS, {
        pollInterval: shouldPoll ? 2000 : 0,
        errorPolicy: 'none',
        onError: () => setShouldPoll(false)
    });

    // 2. Mutation Hooks
    const [createProjectData, { loading: creating }] = useMutation(CREATE_PROJECT);
    const [loadProjectData, { loading: loadingProject }] = useMutation(LOAD_PROJECT);
    const [deleteProjectData] = useMutation(DELETE_PROJECT);
    const [exportProjectData, { loading: exporting }] = useMutation(EXPORT_PROJECT);
    const [importProjectData, { loading: importing }] = useMutation(IMPORT_PROJECT);

    // 3. Early Return: Connection Error
    if (connectionError) {
        return <OrchestratorConnectionError onRetry={() => {
            setShouldPoll(true);
            refetch();
        }} />;
    }

    // Handlers
    const handleCreate = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!newProjectName.trim()) return;

        // Basit regex validasyonu (sadece alfanümerik ve tire)
        if (!/^[a-zA-Z0-9_-]+$/.test(newProjectName)) {
            setError("Project name can only contain letters, numbers, hyphens and underscores.");
            return;
        }

        try {
            setError(null);
            const res = await createProjectData({ variables: { name: newProjectName } });
            if (res.data?.createProject?.success) {
                await refetch();
                setNewProjectName('');
                // İsteğe bağlı: Oluşturunca direkt yükle
                handleLoad(newProjectName);
            } else {
                setError("Failed to create project");
            }
        } catch (err: any) {
            setError(err.message);
        }
    };

    const handleLoad = async (name: string) => {
        try {
            const res = await loadProjectData({ variables: { name } });
            if (res.data?.loadProject?.success) {
                navigate('/');
            }
        } catch (err: any) {
            setError(err.message);
        }
    };

    const handleDelete = async (e: React.MouseEvent, name: string) => {
        e.stopPropagation(); // Don't trigger handleLoad
        if (!window.confirm(`Are you sure you want to delete project '${name}'? All data will be lost.`)) {
            return;
        }

        try {
            const res = await deleteProjectData({ variables: { name } });
            if (res.data?.deleteProject?.success) {
                await refetch();
            }
        } catch (err: any) {
            setError(err.message);
        }
    };

    const handleExport = async (e: React.MouseEvent, name: string) => {
        e.stopPropagation();
        try {
            const outputPath = await save({
                filters: [{ name: 'Proxxy Project', extensions: ['proxxy'] }],
                defaultPath: `${name}.proxxy`
            });

            if (outputPath) {
                const res = await exportProjectData({
                    variables: { name, outputPath }
                });
                if (res.data?.exportProject?.success) {
                    alert(`Project exported successfully to ${outputPath}`);
                } else {
                    setError(res.data?.exportProject?.message || "Export failed");
                }
            }
        } catch (err: any) {
            setError(err.message);
        }
    };

    const handleImport = async () => {
        try {
            const selected = await open({
                filters: [{ name: 'Proxxy Project', extensions: ['proxxy'] }],
                multiple: false,
            });

            if (selected && typeof selected === 'string') {
                const res = await importProjectData({
                    variables: { proxxyPath: selected }
                });
                if (res.data?.importProject?.success) {
                    await refetch();
                    alert("Project imported successfully!");
                } else {
                    setError(res.data?.importProject?.message || "Import failed");
                }
            }
        } catch (err: any) {
            setError(err.message);
        }
    };

    // Data Processing & Filtering
    const allProjects = data?.projects || [];
    const filteredProjects = allProjects.filter((p: any) =>
        p.name.toLowerCase().includes(searchTerm.toLowerCase())
    );

    // Helpers
    const formatSize = (bytes: number) => {
        if (bytes === 0) return 'Empty';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
    };

    const formatTime = (iso: string) => {
        return new Date(iso).toLocaleDateString('en-US', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
    }

    return (
        <div className="min-h-screen bg-[#0B0D11] text-slate-200 font-sans selection:bg-emerald-500/30 flex items-center justify-center p-4 md:p-8">

            <div className="w-full max-w-6xl space-y-8">

                {/* Header Section */}
                <div data-tauri-drag-region className="flex flex-col md:flex-row md:items-end justify-between gap-4 border-b border-white/5 pb-6">
                    <div>
                        <h1 className="text-3xl md:text-4xl font-bold text-white tracking-tight mb-2 flex items-center gap-3">
                            <ShieldCheck className="w-8 h-8 text-emerald-400" />
                            <span className="bg-clip-text text-transparent bg-gradient-to-r from-white to-slate-400">
                                Proxxy Workspace
                            </span>
                        </h1>
                        <p className="text-slate-400 text-sm md:text-base max-w-xl">
                            Select a project database to initialize the interception orchestrator.
                            Each workspace is fully isolated.
                        </p>
                    </div>
                    <div className="text-right hidden md:block">
                        <div className="text-xs font-mono text-emerald-500 bg-emerald-500/10 px-3 py-1 rounded-full border border-emerald-500/20 inline-block">
                            SYSTEM ONLINE
                        </div>
                    </div>
                </div>

                {/* Main Grid Layout */}
                <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 md:gap-8 h-auto lg:h-[600px]">

                    {/* COL 1: CREATE PROJECT (LG: 4 cols) */}
                    <div className="lg:col-span-4 bg-[#111318] border border-white/5 rounded-2xl p-6 flex flex-col shadow-2xl relative overflow-hidden group">
                        {/* Decorative Background Glow */}
                        <div className="absolute top-0 right-0 w-64 h-64 bg-emerald-500/5 rounded-full blur-3xl -mr-16 -mt-16 pointer-events-none transition-opacity group-hover:opacity-100 opacity-50" />

                        <div className="flex items-center justify-between mb-6 relative z-10">
                            <div className="flex items-center space-x-3">
                                <div className="p-2 bg-emerald-500/10 rounded-lg border border-emerald-500/20">
                                    <Plus className="w-5 h-5 text-emerald-400" />
                                </div>
                                <h2 className="text-lg font-semibold text-white">New Workspace</h2>
                            </div>

                            <button
                                onClick={handleImport}
                                disabled={importing}
                                className="p-2 bg-blue-500/10 hover:bg-blue-500/20 rounded-lg border border-blue-500/20 text-blue-400 transition-colors flex items-center gap-2 text-xs font-bold"
                                title="Import .proxxy file"
                            >
                                {importing ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Upload className="w-3.5 h-3.5" />}
                                IMPORT
                            </button>
                        </div>

                        <form onSubmit={handleCreate} className="flex-1 flex flex-col relative z-10">
                            <div className="space-y-4">
                                <div>
                                    <label className="block text-xs font-bold text-slate-500 uppercase tracking-wider mb-2">
                                        Project Name
                                    </label>
                                    <input
                                        type="text"
                                        className="w-full bg-black/40 border border-white/10 rounded-xl px-4 py-3 text-white placeholder:text-slate-600 focus:ring-2 focus:ring-emerald-500/20 focus:border-emerald-500 outline-none transition-all font-mono text-sm"
                                        placeholder="client_pentest_v1"
                                        value={newProjectName}
                                        onChange={e => setNewProjectName(e.target.value)}
                                        maxLength={30}
                                    />
                                    <p className="text-[10px] text-slate-500 mt-2 ml-1">
                                        Allowed: a-z, 0-9, underscores, hyphens.
                                    </p>
                                </div>

                                {error && (
                                    <div className="flex items-start gap-2 bg-red-500/10 border border-red-500/20 rounded-lg p-3 text-red-400 text-xs">
                                        <AlertCircle className="w-4 h-4 shrink-0 mt-0.5" />
                                        <span>{error}</span>
                                    </div>
                                )}
                            </div>

                            <div className="mt-auto pt-6">
                                <button
                                    type="submit"
                                    disabled={creating || !newProjectName}
                                    className="w-full py-3.5 px-4 bg-gradient-to-r from-emerald-600 to-emerald-500 hover:from-emerald-500 hover:to-emerald-400 disabled:from-slate-800 disabled:to-slate-800 disabled:text-slate-500 disabled:cursor-not-allowed text-white rounded-xl font-bold text-sm tracking-wide shadow-lg shadow-emerald-900/20 transition-all flex items-center justify-center space-x-2 group/btn"
                                >
                                    {creating ? (
                                        <Loader2 className="w-4 h-4 animate-spin" />
                                    ) : (
                                        <>
                                            <Database className="w-4 h-4" />
                                            <span>INITIALIZE DB</span>
                                            <ArrowRight className="w-4 h-4 opacity-50 group-hover/btn:translate-x-1 transition-transform" />
                                        </>
                                    )}
                                </button>
                            </div>
                        </form>
                    </div>

                    {/* COL 2: PROJECT LIST (LG: 8 cols) */}
                    <div className="lg:col-span-8 bg-[#111318] border border-white/5 rounded-2xl flex flex-col shadow-2xl overflow-hidden">

                        {/* List Header & Search */}
                        <div className="p-6 border-b border-white/5 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
                            <div className="flex items-center space-x-3">
                                <div className="p-2 bg-blue-500/10 rounded-lg border border-blue-500/20">
                                    <Folder className="w-5 h-5 text-blue-400" />
                                </div>
                                <div>
                                    <h2 className="text-lg font-semibold text-white">Recent Projects</h2>
                                    <p className="text-xs text-slate-500">{allProjects.length} databases found</p>
                                </div>
                            </div>

                            <div className="relative group">
                                <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-slate-500 group-focus-within:text-blue-400 transition-colors" />
                                <input
                                    type="text"
                                    placeholder="Search projects..."
                                    value={searchTerm}
                                    onChange={(e) => setSearchTerm(e.target.value)}
                                    className="bg-black/40 border border-white/10 rounded-xl py-2 pl-10 pr-4 text-sm text-white placeholder:text-slate-600 focus:border-blue-500/50 focus:ring-1 focus:ring-blue-500/20 outline-none w-full sm:w-64 transition-all"
                                />
                            </div>
                        </div>

                        {/* Scrollable List */}
                        <div className="flex-1 overflow-y-auto p-4 space-y-2 custom-scrollbar">
                            {loading && allProjects.length === 0 ? (
                                <div className="h-full flex flex-col items-center justify-center text-slate-600 gap-3">
                                    <Loader2 className="w-8 h-8 animate-spin" />
                                    <span className="text-sm">Scanning databases...</span>
                                </div>
                            ) : filteredProjects.length === 0 ? (
                                <div className="h-full flex flex-col items-center justify-center text-slate-600 gap-3 min-h-[300px]">
                                    <Folder className="w-12 h-12 opacity-20" />
                                    <p className="text-sm">No matching projects found.</p>
                                </div>
                            ) : (
                                filteredProjects.map((p: any) => (
                                    <div
                                        key={p.name}
                                        onClick={() => handleLoad(p.name)}
                                        className={`group flex items-center justify-between p-4 rounded-xl border transition-all cursor-pointer relative overflow-hidden ${p.isActive
                                            ? 'bg-emerald-500/[0.03] border-emerald-500/30 hover:bg-emerald-500/[0.05]'
                                            : 'bg-white/[0.02] border-white/5 hover:border-blue-500/30 hover:bg-white/[0.04]'
                                            }`}
                                    >
                                        {/* Active Indicator Strip */}
                                        {p.isActive && <div className="absolute left-0 top-0 bottom-0 w-1 bg-emerald-500" />}

                                        <div className="flex items-center gap-4">
                                            {/* Icon */}
                                            <div className={`p-3 rounded-lg ${p.isActive ? 'bg-emerald-500/10 text-emerald-400' : 'bg-white/5 text-slate-500 group-hover:text-blue-400'
                                                } transition-colors`}>
                                                <Database className="w-5 h-5" />
                                            </div>

                                            {/* Details */}
                                            <div>
                                                <div className="flex items-center gap-2">
                                                    <h3 className={`font-semibold text-sm ${p.isActive ? 'text-emerald-400' : 'text-slate-200 group-hover:text-white'
                                                        } transition-colors`}>
                                                        {p.name}
                                                    </h3>
                                                    {p.isActive && (
                                                        <span className="text-[10px] font-bold bg-emerald-500/10 text-emerald-400 px-2 py-0.5 rounded-full border border-emerald-500/20">
                                                            ACTIVE
                                                        </span>
                                                    )}
                                                </div>

                                                <div className="flex items-center gap-4 mt-1.5">
                                                    <div className="flex items-center gap-1.5 text-xs text-slate-500">
                                                        <HardDrive className="w-3 h-3" />
                                                        <span>{formatSize(p.sizeBytes)}</span>
                                                    </div>
                                                    <div className="flex items-center gap-1.5 text-xs text-slate-500">
                                                        <Clock className="w-3 h-3" />
                                                        <span>{formatTime(p.lastModified)}</span>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        {/* Action Buttons */}
                                        <div className="flex items-center gap-2 pr-2">
                                            <button
                                                onClick={(e) => handleExport(e, p.name)}
                                                disabled={exporting}
                                                className="p-2 rounded-lg bg-emerald-500/10 text-emerald-400 hover:bg-emerald-500 hover:text-white transition-all opacity-0 group-hover:opacity-100"
                                                title="Export to .proxxy file"
                                            >
                                                {exporting ? <Loader2 className="w-4 h-4 animate-spin" /> : <Download className="w-4 h-4" />}
                                            </button>

                                            <button
                                                onClick={(e) => handleDelete(e, p.name)}
                                                className="p-2 rounded-lg bg-red-500/10 text-red-400 hover:bg-red-500 hover:text-white transition-all opacity-0 group-hover:opacity-100"
                                                title="Delete Project"
                                            >
                                                <Trash2 className="w-4 h-4" />
                                            </button>

                                            <button
                                                disabled={loadingProject}
                                                className={`p-2 rounded-lg transition-all ${p.isActive
                                                    ? 'bg-emerald-500/10 text-emerald-400'
                                                    : 'bg-white/5 text-slate-400 group-hover:bg-blue-500 group-hover:text-white opacity-0 group-hover:opacity-100 transform translate-x-2 group-hover:translate-x-0'
                                                    }`}
                                            >
                                                {loadingProject && p.name === newProjectName ? (
                                                    <Loader2 className="w-4 h-4 animate-spin" />
                                                ) : (
                                                    <ArrowRight className="w-4 h-4" />
                                                )}
                                            </button>
                                        </div>
                                    </div>
                                ))
                            )}
                        </div>

                        {/* Footer Info */}
                        <div className="p-4 bg-black/20 border-t border-white/5 text-[10px] text-slate-500 font-mono flex justify-between">
                            <span>STORAGE: SQLITE (WAL MODE)</span>
                            <span>AVAILABLE: {allProjects.length} PROJECTS</span>
                        </div>
                    </div>

                </div>
            </div>

            {/* Global Custom Scrollbar Style */}
            <style>{`
                .custom-scrollbar::-webkit-scrollbar {
                    width: 6px;
                }
                .custom-scrollbar::-webkit-scrollbar-track {
                    background: rgba(255, 255, 255, 0.02);
                }
                .custom-scrollbar::-webkit-scrollbar-thumb {
                    background: rgba(255, 255, 255, 0.1);
                    border-radius: 10px;
                }
                .custom-scrollbar::-webkit-scrollbar-thumb:hover {
                    background: rgba(255, 255, 255, 0.2);
                }
            `}</style>
        </div>
    );
}