import React, { useState } from 'react';
import { useQuery, useMutation } from '@apollo/client';
import {
    Folder, Plus, Database, HardDrive, Clock,
    ArrowRight, Loader2, Search, ShieldCheck,
    AlertCircle, Trash2, Download, Upload,
    ChevronRight, Activity
} from 'lucide-react';
import {
    GET_PROJECTS, CREATE_PROJECT, LOAD_PROJECT,
    DELETE_PROJECT, EXPORT_PROJECT, IMPORT_PROJECT
} from '@/graphql/operations';
import { useNavigate } from 'react-router-dom';
import { OrchestratorConnectionError } from '@/components/OrchestratorConnectionError';
import { open, save } from '@tauri-apps/plugin-dialog';

// Shadcn UI Components
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardHeader, CardTitle, CardDescription, CardFooter } from "@/components/ui/card";

export function ProjectLauncher() {
    const navigate = useNavigate();
    const [newProjectName, setNewProjectName] = useState('');
    const [searchTerm, setSearchTerm] = useState('');
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
        e.stopPropagation();
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

    const allProjects = data?.projects || [];
    const filteredProjects = allProjects.filter((p: any) =>
        p.name.toLowerCase().includes(searchTerm.toLowerCase())
    );

    const formatSize = (bytes: number) => {
        if (bytes === 0) return 'Empty';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
    };

    const formatTime = (iso: string) => {
        return new Date(iso).toLocaleDateString('en-US', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
    };

    return (
        <div className="min-h-screen bg-background text-foreground p-4 sm:p-6 md:p-10 flex items-center justify-center transition-colors">

            {/* Background Glows */}
            <div className="fixed inset-0 overflow-hidden pointer-events-none opacity-20">
                <div className="absolute top-[-10%] right-[-10%] w-[40%] h-[40%] bg-primary blur-[120px] rounded-full" />
                <div className="absolute bottom-[-10%] left-[-10%] w-[30%] h-[30%] bg-blue-500 blur-[100px] rounded-full" />
            </div>

            <div className="w-full max-w-6xl z-10 space-y-8 animate-in fade-in duration-700">

                {/* Header Section */}
                <header className="flex flex-col md:flex-row md:items-center justify-between gap-6">
                    <div className="space-y-1">
                        <div className="flex items-center gap-3">
                            <div className="p-2.5 bg-primary/10 rounded-xl border border-primary/20 shadow-inner">
                                <ShieldCheck className="w-7 h-7 text-primary" />
                            </div>
                            <h1 className="text-3xl md:text-4xl font-extrabold tracking-tight">
                                Proxxy<span className="text-primary">.</span>
                            </h1>
                        </div>
                        <p className="text-muted-foreground text-sm md:text-base font-medium max-w-lg leading-relaxed pt-2">
                            Orchestrate your interception workflows with isolated workspaces.
                        </p>
                    </div>

                    <div className="flex items-center gap-3">
                        <div className="hidden sm:flex items-center gap-2.5 px-3 py-1.5 bg-muted/30 border border-border rounded-full">
                            <div className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
                            <span className="text-[10px] font-bold uppercase tracking-widest">System Live</span>
                        </div>
                        <Button variant="outline" size="sm" onClick={handleImport} disabled={importing}>
                            {importing ? <Loader2 className="w-4 h-4 mr-2 animate-spin" /> : <Upload className="w-4 h-4 mr-2" />}
                            Import Project
                        </Button>
                    </div>
                </header>

                {/* Content Grid */}
                <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 items-start">

                    {/* Create Project Section */}
                    <aside className="lg:col-span-4 space-y-6">
                        <Card className="shadow-2xl border-border/50 overflow-hidden">
                            <CardHeader className="pb-4">
                                <CardTitle className="flex items-center gap-2 text-lg">
                                    <Plus className="w-5 h-5 text-primary" />
                                    New Project
                                </CardTitle>
                                <CardDescription>Initialize a fresh database workspace.</CardDescription>
                            </CardHeader>
                            <CardContent>
                                <form onSubmit={handleCreate} className="space-y-4">
                                    <div className="space-y-2">
                                        <label className="text-[10px] font-bold text-muted-foreground uppercase tracking-widest ml-1">
                                            Project Hash
                                        </label>
                                        <div className="relative">
                                            <Database className="absolute left-3.5 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
                                            <Input
                                                className="pl-10 font-mono text-xs"
                                                placeholder="pentest_alpha"
                                                value={newProjectName}
                                                onChange={e => setNewProjectName(e.target.value)}
                                                maxLength={30}
                                            />
                                        </div>
                                    </div>

                                    {error && (
                                        <div className="flex items-start gap-3 bg-destructive/10 border border-destructive/20 rounded-lg p-3 text-destructive text-[10px] font-bold animate-in zoom-in-95">
                                            <AlertCircle className="w-3.5 h-3.5 mt-0.5 shrink-0" />
                                            <span>{error}</span>
                                        </div>
                                    )}

                                    <Button
                                        type="submit"
                                        className="w-full font-bold tracking-widest active:scale-95"
                                        disabled={creating || !newProjectName}
                                    >
                                        {creating ? <Loader2 className="w-4 h-4 animate-spin" /> : <>CREATE WORKSPACE <ChevronRight className="w-4 h-4 ml-1" /></>}
                                    </Button>
                                </form>
                            </CardContent>
                            <CardFooter className="bg-muted/30 border-t py-3">
                                <div className="flex items-center gap-2 text-[10px] text-muted-foreground font-medium">
                                    <Activity className="w-3 h-3 text-emerald-500" />
                                    Isolation level: Fully Encrypted Persistence
                                </div>
                            </CardFooter>
                        </Card>
                    </aside>

                    {/* Project Explorer Section */}
                    <main className="lg:col-span-8">
                        <Card className="shadow-2xl border-border/50 flex flex-col min-h-[500px]">
                            <CardHeader className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b pb-4 bg-muted/10">
                                <div className="space-y-1">
                                    <CardTitle className="flex items-center gap-2">
                                        <Folder className="w-5 h-5 text-primary" />
                                        Project Explorer
                                    </CardTitle>
                                    <CardDescription className="text-xs">
                                        {filteredProjects.length} projects identified.
                                    </CardDescription>
                                </div>
                                <div className="relative">
                                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
                                    <Input
                                        placeholder="Filter..."
                                        value={searchTerm}
                                        onChange={(e) => setSearchTerm(e.target.value)}
                                        className="pl-9 w-full sm:w-[240px] h-9 text-xs"
                                    />
                                </div>
                            </CardHeader>

                            <CardContent className="p-4 md:p-6 space-y-3 flex-1 overflow-y-auto custom-scrollbar h-[450px]">
                                {loading && allProjects.length === 0 ? (
                                    <div className="h-full flex flex-col items-center justify-center text-muted-foreground gap-4 py-32">
                                        <Loader2 className="w-8 h-8 animate-spin text-primary" />
                                        <span className="text-[10px] font-bold uppercase tracking-[0.2em]">Synchronizing Registry...</span>
                                    </div>
                                ) : filteredProjects.length === 0 ? (
                                    <div className="h-full flex flex-col items-center justify-center text-muted-foreground/30 gap-4 py-32 border-2 border-dashed rounded-2xl">
                                        <Folder className="w-12 h-12" />
                                        <p className="text-xs font-medium uppercase tracking-widest text-muted-foreground/50">No workspaces found.</p>
                                    </div>
                                ) : (
                                    filteredProjects.map((p: any) => (
                                        <div
                                            key={p.name}
                                            onClick={() => handleLoad(p.name)}
                                            className={`group relative grid grid-cols-1 sm:grid-cols-[auto_1fr_auto] items-center gap-4 p-4 rounded-xl border transition-all cursor-pointer ${p.isActive
                                                    ? 'bg-primary/5 border-primary/40 shadow-[0_0_20px_rgba(var(--primary),0.05)]'
                                                    : 'bg-muted/5 border-border/50 hover:bg-muted/20 hover:border-primary/20 hover:scale-[1.01]'
                                                }`}
                                        >
                                            {/* Active Stripe */}
                                            {p.isActive && <div className="absolute left-0 top-0 bottom-0 w-1 bg-primary rounded-l-xl" />}

                                            {/* Icon */}
                                            <div className={`p-3 rounded-lg border ${p.isActive ? 'bg-primary/10 border-primary/20 text-primary' : 'bg-muted/50 border-border text-muted-foreground group-hover:text-primary transition-colors'
                                                }`}>
                                                <Database className="w-5 h-5" />
                                            </div>

                                            {/* Info */}
                                            <div className="min-w-0 pr-4">
                                                <div className="flex items-center gap-2 mb-1">
                                                    <h3 className={`font-bold text-sm truncate uppercase tracking-tighter ${p.isActive ? 'text-primary' : 'text-foreground'}`}>
                                                        {p.name}
                                                    </h3>
                                                    {p.isActive && (
                                                        <span className="text-[8px] font-black bg-primary text-primary-foreground px-1.5 py-0.5 rounded uppercase shadow-sm">
                                                            Active
                                                        </span>
                                                    )}
                                                </div>
                                                <div className="flex items-center gap-4">
                                                    <span className="flex items-center gap-1 text-[10px] font-bold text-muted-foreground uppercase opacity-70 group-hover:opacity-100 transition-opacity">
                                                        <HardDrive className="w-3 h-3" /> {formatSize(p.sizeBytes)}
                                                    </span>
                                                    <span className="flex items-center gap-1 text-[10px] font-bold text-muted-foreground uppercase opacity-70 group-hover:opacity-100 transition-opacity">
                                                        <Clock className="w-3 h-3" /> {formatTime(p.lastModified)}
                                                    </span>
                                                </div>
                                            </div>

                                            {/* Actions */}
                                            <div className="flex items-center gap-1.5 opacity-0 group-hover:opacity-100 translate-x-2 group-hover:translate-x-0 transition-all duration-300">
                                                <Button variant="ghost" size="icon" className="h-8 w-8 text-muted-foreground hover:text-emerald-500 hover:bg-emerald-500/10" onClick={(e) => handleExport(e, p.name)} disabled={exporting}>
                                                    {exporting ? <Loader2 className="w-4 h-4 animate-spin" /> : <Download className="w-4 h-4" />}
                                                </Button>
                                                <Button variant="ghost" size="icon" className="h-8 w-8 text-muted-foreground hover:text-destructive hover:bg-destructive/10" onClick={(e) => handleDelete(e, p.name)}>
                                                    <Trash2 className="w-4 h-4" />
                                                </Button>
                                                <div className={`p-1.5 rounded-lg border transition-all ${p.isActive ? 'bg-primary/20 border-primary/20 text-primary' : 'bg-primary text-primary-foreground shadow-lg'}`}>
                                                    {loadingProject && p.name === newProjectName ? <Loader2 className="w-4 h-4 animate-spin" /> : <ArrowRight className="w-4 h-4" />}
                                                </div>
                                            </div>
                                        </div>
                                    ))
                                )}
                            </CardContent>
                            <CardFooter className="bg-muted/10 border-t py-2 flex justify-between px-6">
                                <span className="text-[9px] font-black text-muted-foreground/40 uppercase tracking-widest">Storage: WAL-Optimized SQLite</span>
                                <span className="text-[9px] font-black text-muted-foreground/40 uppercase tracking-widest">Layer: Orchestrator Registry</span>
                            </CardFooter>
                        </Card>
                    </main>
                </div>
            </div>

            <style>{`
        .custom-scrollbar::-webkit-scrollbar { width: 4px; }
        .custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
        .custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(var(--primary), 0.1); border-radius: 10px; }
        .custom-scrollbar::-webkit-scrollbar-thumb:hover { background: rgba(var(--primary), 0.3); }
      `}</style>
        </div>
    );
}