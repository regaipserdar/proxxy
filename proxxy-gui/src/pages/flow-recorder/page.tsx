import { useState, useEffect } from 'react';
import { useQuery, useMutation } from '@apollo/client';
import {
    Play,
    Trash2,
    Plus,
    RefreshCw,
    Clock,
    CheckCircle,
    XCircle,
    Video,
    Circle,
    Square,
    Bug,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from '@/components/ui/dialog';
import {
    GET_FLOW_PROFILES,
    GET_FLOW_EXECUTIONS,
    CREATE_FLOW_PROFILE,
    DELETE_FLOW_PROFILE,
    REPLAY_FLOW,
    GET_AGENTS,
    START_FLOW_RECORDING,
    STOP_FLOW_RECORDING,
    DEBUG_LAUNCH_BROWSER,
    DEBUG_CLOSE_BROWSER
} from '@/graphql/operations';

interface FlowProfile {
    id: string;
    name: string;
    flowType: string;
    startUrl: string;
    status: string;
    stepCount: number;
    createdAt: number;
    updatedAt: number;
    agentId?: string;
}

interface FlowExecution {
    id: string;
    profileId: string;
    agentId: string;
    startedAt: number;
    completedAt?: number;
    status: string;
    errorMessage?: string;
    stepsCompleted: number;
    totalSteps: number;
    durationMs?: number;
}

const FlowTypeColors: Record<string, string> = {
    Login: 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30',
    Checkout: 'bg-amber-500/20 text-amber-400 border-amber-500/30',
    FormSubmission: 'bg-blue-500/20 text-blue-400 border-blue-500/30',
    Navigation: 'bg-purple-500/20 text-purple-400 border-purple-500/30',
    Custom: 'bg-slate-500/20 text-slate-400 border-slate-500/30',
};

const StatusIcons: Record<string, React.ReactNode> = {
    Active: <CheckCircle size={14} className="text-emerald-400" />,
    Recording: <Video size={14} className="text-red-400 animate-pulse" />,
    Archived: <Clock size={14} className="text-slate-400" />,
    Failed: <XCircle size={14} className="text-red-400" />,
};

const FLOW_TYPES = ['Login', 'Checkout', 'FormSubmission', 'Navigation', 'Custom'];

export const FlowRecorderView = () => {
    const [selectedProfileId, setSelectedProfileId] = useState<string | null>(null);
    const [isCreateOpen, setIsCreateOpen] = useState(false);
    const [isRecordOpen, setIsRecordOpen] = useState(false);
    const [isRecording, setIsRecording] = useState(false);
    const [recordingProfileId, setRecordingProfileId] = useState<string | null>(null);
    const [newProfileName, setNewProfileName] = useState('');
    const [newProfileUrl, setNewProfileUrl] = useState('');
    const [newProfileType, setNewProfileType] = useState('Login');
    const [recordName, setRecordName] = useState('');
    const [recordUrl, setRecordUrl] = useState('https://');
    const [isDebugBrowserOpen, setIsDebugBrowserOpen] = useState(false);
    const [debugUrl, setDebugUrl] = useState('https://example.com');

    // GraphQL Queries
    const { data: profilesData, loading: profilesLoading, refetch: refetchProfiles } = useQuery(GET_FLOW_PROFILES, {
        pollInterval: 10000,
    });

    const { data: executionsData, loading: executionsLoading } = useQuery(GET_FLOW_EXECUTIONS, {
        variables: { profileId: selectedProfileId, limit: 10 },
        skip: !selectedProfileId,
    });

    // GraphQL Mutations
    const [createProfile] = useMutation(CREATE_FLOW_PROFILE);
    const [deleteProfile] = useMutation(DELETE_FLOW_PROFILE);
    const [replayFlowMutation] = useMutation(REPLAY_FLOW);
    const [startRecordingMutation] = useMutation(START_FLOW_RECORDING);
    const [stopRecordingMutation] = useMutation(STOP_FLOW_RECORDING);
    const [debugLaunchBrowserMutation] = useMutation(DEBUG_LAUNCH_BROWSER);
    const [debugCloseBrowserMutation] = useMutation(DEBUG_CLOSE_BROWSER);

    // Get available agents for replay
    const { data: agentsData } = useQuery(GET_AGENTS);
    const agents = agentsData?.agents || [];

    const profiles: FlowProfile[] = profilesData?.flowProfiles || [];
    const executions: FlowExecution[] = executionsData?.flowExecutions || [];

    const selectedProfile = profiles.find(p => p.id === selectedProfileId);

    useEffect(() => {
        if (profiles.length > 0 && !selectedProfileId) {
            setSelectedProfileId(profiles[0].id);
        }
    }, [profiles, selectedProfileId]);

    // Start Recording Handler
    const handleStartRecording = async () => {
        if (!recordName.trim() || !recordUrl.trim()) return;

        try {
            const result = await startRecordingMutation({
                variables: {
                    input: {
                        name: recordName,
                        startUrl: recordUrl,
                        flowType: 'Login',
                    }
                }
            });

            if (result.data?.startFlowRecording?.success) {
                setIsRecording(true);
                setRecordingProfileId(result.data.startFlowRecording.profileId);
                setIsRecordOpen(false);
                refetchProfiles();
            } else {
                alert(`Failed to start recording: ${result.data?.startFlowRecording?.message}`);
            }
        } catch (error) {
            console.error('Failed to start recording:', error);
            alert('Failed to start recording');
        }
    };

    // Stop Recording Handler
    const handleStopRecording = async (save: boolean) => {
        if (!recordingProfileId) return;

        try {
            await stopRecordingMutation({
                variables: {
                    profileId: recordingProfileId,
                    input: { save }
                }
            });
            setIsRecording(false);
            setRecordingProfileId(null);
            setRecordName('');
            setRecordUrl('https://');
            refetchProfiles();
        } catch (error) {
            console.error('Failed to stop recording:', error);
        }
    };

    // Debug Browser Handlers
    const handleDebugLaunchBrowser = async () => {
        try {
            const result = await debugLaunchBrowserMutation({
                variables: { startUrl: debugUrl }
            });
            if (result.data?.debugLaunchBrowser?.success) {
                setIsDebugBrowserOpen(true);
                alert('Debug browser launched! Check orchestrator terminal for proxy logs.');
            } else {
                alert(`Failed: ${result.data?.debugLaunchBrowser?.message}`);
            }
        } catch (error) {
            console.error('Failed to launch debug browser:', error);
            alert(`Error: ${error}`);
        }
    };

    const handleDebugCloseBrowser = async () => {
        try {
            await debugCloseBrowserMutation();
            setIsDebugBrowserOpen(false);
            alert('Debug browser closed.');
        } catch (error) {
            console.error('Failed to close debug browser:', error);
        }
    };

    const handleCreateProfile = async () => {
        if (!newProfileName.trim() || !newProfileUrl.trim()) return;

        try {
            await createProfile({
                variables: {
                    input: {
                        name: newProfileName,
                        startUrl: newProfileUrl,
                        flowType: newProfileType,
                    }
                }
            });
            setIsCreateOpen(false);
            setNewProfileName('');
            setNewProfileUrl('');
            refetchProfiles();
        } catch (error) {
            console.error('Failed to create profile:', error);
        }
    };

    const handleDeleteProfile = async (id: string) => {
        if (!confirm('Are you sure you want to delete this flow profile?')) return;

        try {
            await deleteProfile({ variables: { id } });
            if (selectedProfileId === id) {
                setSelectedProfileId(null);
            }
            refetchProfiles();
        } catch (error) {
            console.error('Failed to delete profile:', error);
        }
    };

    const handleReplayFlow = async (profileId: string) => {
        if (agents.length === 0) {
            alert('No agents available for replay');
            return;
        }

        // Use first online agent
        const onlineAgent = agents.find((a: any) => a.status?.toLowerCase() === 'online');
        if (!onlineAgent) {
            alert('No online agent available');
            return;
        }

        try {
            const result = await replayFlowMutation({
                variables: {
                    input: {
                        profileId,
                        agentId: onlineAgent.id,
                        headed: true,
                    }
                }
            });

            if (result.data?.replayFlow?.success) {
                alert(`Replay started! Execution ID: ${result.data.replayFlow.executionId}`);
                // Refetch executions to show new one
                refetchProfiles();
            } else {
                alert(`Replay failed: ${result.data?.replayFlow?.error || 'Unknown error'}`);
            }
        } catch (error) {
            console.error('Failed to replay flow:', error);
            alert('Failed to replay flow');
        }
    };

    const formatDate = (timestamp: number) => {
        return new Date(timestamp * 1000).toLocaleString();
    };

    const formatDuration = (ms?: number) => {
        if (!ms) return '-';
        if (ms < 1000) return `${ms}ms`;
        return `${(ms / 1000).toFixed(2)}s`;
    };

    return (
        <div className="flex flex-col h-full bg-[#0A0E14] text-white/80 font-sans overflow-hidden">
            {/* Header */}
            <div className="flex items-center justify-between px-4 py-3 border-b border-white/5 bg-gradient-to-r from-cyan-500/5 to-transparent">
                <div className="flex items-center gap-3">
                    <Video className="text-cyan-400" size={20} />
                    <h1 className="text-lg font-semibold text-white/90">Flow Recorder</h1>
                    <Badge variant="outline" className="text-xs border-cyan-500/30 text-cyan-400">
                        {profiles.length} profiles
                    </Badge>
                </div>
                <div className="flex items-center gap-2">
                    {/* Recording Status Bar */}
                    {isRecording && (
                        <div className="flex items-center gap-2 px-3 py-1 bg-red-500/20 border border-red-500/30 rounded-md">
                            <Circle size={10} className="text-red-400 animate-pulse fill-red-400" />
                            <span className="text-xs text-red-400">Recording...</span>
                            <Button
                                size="sm"
                                variant="ghost"
                                onClick={() => handleStopRecording(true)}
                                className="text-emerald-400 hover:bg-emerald-500/10 px-2 h-6"
                            >
                                Save
                            </Button>
                            <Button
                                size="sm"
                                variant="ghost"
                                onClick={() => handleStopRecording(false)}
                                className="text-red-400 hover:bg-red-500/10 px-2 h-6"
                            >
                                <Square size={12} />
                            </Button>
                        </div>
                    )}
                    <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => refetchProfiles()}
                        className="text-slate-400 hover:text-white"
                    >
                        <RefreshCw size={14} />
                    </Button>
                    {/* Debug Browser Section */}
                    <div className="flex items-center gap-1 px-2 py-1 bg-yellow-500/10 border border-yellow-500/30 rounded-md">
                        <Bug size={14} className="text-yellow-400" />
                        <Input
                            value={debugUrl}
                            onChange={(e) => setDebugUrl(e.target.value)}
                            className="w-48 h-6 text-xs bg-transparent border-none text-white/80"
                            placeholder="https://..."
                        />
                        {!isDebugBrowserOpen ? (
                            <Button
                                size="sm"
                                variant="ghost"
                                onClick={handleDebugLaunchBrowser}
                                className="text-yellow-400 hover:bg-yellow-500/10 px-2 h-6 text-xs"
                            >
                                Launch
                            </Button>
                        ) : (
                            <Button
                                size="sm"
                                variant="ghost"
                                onClick={handleDebugCloseBrowser}
                                className="text-red-400 hover:bg-red-500/10 px-2 h-6 text-xs"
                            >
                                Close
                            </Button>
                        )}
                    </div>
                    {/* Start Recording Dialog */}
                    <Dialog open={isRecordOpen} onOpenChange={setIsRecordOpen}>
                        <DialogTrigger asChild>
                            <Button
                                size="sm"
                                className="bg-red-500/20 text-red-400 hover:bg-red-500/30 border border-red-500/30"
                                disabled={isRecording}
                            >
                                <Circle size={14} className="mr-1 fill-red-400" /> Record
                            </Button>
                        </DialogTrigger>
                        <DialogContent className="bg-[#0D1117] border-white/10 text-white">
                            <DialogHeader>
                                <DialogTitle>Start Browser Recording</DialogTitle>
                                <DialogDescription className="text-slate-400">
                                    A browser window will open with Proxxy proxy configured.
                                    Perform your login/flow actions, then save the recording.
                                </DialogDescription>
                            </DialogHeader>
                            <div className="space-y-4 py-4">
                                <div>
                                    <label className="text-sm text-slate-400 mb-1 block">Flow Name</label>
                                    <Input
                                        placeholder="My Login Flow"
                                        value={recordName}
                                        onChange={(e) => setRecordName(e.target.value)}
                                        className="bg-[#161B22] border-white/10"
                                    />
                                </div>
                                <div>
                                    <label className="text-sm text-slate-400 mb-1 block">Start URL</label>
                                    <Input
                                        placeholder="https://example.com/login"
                                        value={recordUrl}
                                        onChange={(e) => setRecordUrl(e.target.value)}
                                        className="bg-[#161B22] border-white/10"
                                    />
                                </div>
                            </div>
                            <DialogFooter>
                                <Button
                                    onClick={handleStartRecording}
                                    className="bg-red-500 hover:bg-red-600 text-white"
                                    disabled={!recordName.trim() || !recordUrl.trim()}
                                >
                                    <Circle size={14} className="mr-1 fill-white" /> Start Recording
                                </Button>
                            </DialogFooter>
                        </DialogContent>
                    </Dialog>
                    <Dialog open={isCreateOpen} onOpenChange={setIsCreateOpen}>
                        <DialogTrigger asChild>
                            <Button size="sm" className="bg-cyan-500/20 text-cyan-400 hover:bg-cyan-500/30 border border-cyan-500/30">
                                <Plus size={14} className="mr-1" /> New Flow
                            </Button>
                        </DialogTrigger>
                        <DialogContent className="bg-[#0D1117] border-white/10 text-white">
                            <DialogHeader>
                                <DialogTitle>Create New Flow Profile</DialogTitle>
                                <DialogDescription className="text-slate-400">
                                    Define a new browser flow to record and replay.
                                </DialogDescription>
                            </DialogHeader>
                            <div className="grid gap-4 py-4">
                                <div className="grid gap-2">
                                    <label htmlFor="name" className="text-sm text-white/70">Name</label>
                                    <Input
                                        id="name"
                                        value={newProfileName}
                                        onChange={(e) => setNewProfileName(e.target.value)}
                                        placeholder="My Login Flow"
                                        className="bg-[#161B22] border-white/10"
                                    />
                                </div>
                                <div className="grid gap-2">
                                    <label htmlFor="url" className="text-sm text-white/70">Start URL</label>
                                    <Input
                                        id="url"
                                        value={newProfileUrl}
                                        onChange={(e) => setNewProfileUrl(e.target.value)}
                                        placeholder="https://example.com/login"
                                        className="bg-[#161B22] border-white/10"
                                    />
                                </div>
                                <div className="grid gap-2">
                                    <label className="text-sm text-white/70">Flow Type</label>
                                    <div className="flex flex-wrap gap-2">
                                        {FLOW_TYPES.map((type) => (
                                            <button
                                                key={type}
                                                onClick={() => setNewProfileType(type)}
                                                className={`px-3 py-1.5 text-xs rounded-lg border transition-colors ${newProfileType === type
                                                    ? 'bg-cyan-500/20 border-cyan-500/30 text-cyan-400'
                                                    : 'bg-white/5 border-white/10 text-white/60 hover:bg-white/10'
                                                    }`}
                                            >
                                                {type}
                                            </button>
                                        ))}
                                    </div>
                                </div>
                            </div>
                            <DialogFooter>
                                <Button variant="ghost" onClick={() => setIsCreateOpen(false)}>Cancel</Button>
                                <Button onClick={handleCreateProfile} className="bg-cyan-500 hover:bg-cyan-600">
                                    Create Flow
                                </Button>
                            </DialogFooter>
                        </DialogContent>
                    </Dialog>
                </div>
            </div>

            {/* Main Content */}
            <div className="flex-1 flex overflow-hidden">
                {/* Profile List */}
                <div className="w-80 border-r border-white/5 flex flex-col">
                    <div className="p-3 border-b border-white/5">
                        <Input
                            placeholder="Search flows..."
                            className="bg-[#161B22] border-white/10 text-sm"
                        />
                    </div>
                    <div className="flex-1 overflow-y-auto">
                        {profilesLoading ? (
                            <div className="flex items-center justify-center p-8">
                                <div className="w-6 h-6 border-2 border-cyan-500/30 border-t-cyan-500 rounded-full animate-spin" />
                            </div>
                        ) : profiles.length === 0 ? (
                            <div className="flex flex-col items-center justify-center p-8 text-slate-500">
                                <Video size={32} className="mb-3 opacity-50" />
                                <p className="text-sm">No flow profiles yet</p>
                                <p className="text-xs mt-1">Create one to get started</p>
                            </div>
                        ) : (
                            profiles.map((profile) => (
                                <div
                                    key={profile.id}
                                    onClick={() => setSelectedProfileId(profile.id)}
                                    className={`p-3 border-b border-white/5 cursor-pointer transition-colors ${selectedProfileId === profile.id
                                        ? 'bg-cyan-500/10 border-l-2 border-l-cyan-500'
                                        : 'hover:bg-white/5'
                                        }`}
                                >
                                    <div className="flex items-center justify-between mb-1">
                                        <span className="font-medium text-sm text-white/90 truncate">
                                            {profile.name}
                                        </span>
                                        {StatusIcons[profile.status]}
                                    </div>
                                    <div className="flex items-center gap-2 mb-2">
                                        <Badge className={`text-xs ${FlowTypeColors[profile.flowType] || FlowTypeColors.Custom}`}>
                                            {profile.flowType}
                                        </Badge>
                                        <span className="text-xs text-slate-500">
                                            {profile.stepCount} steps
                                        </span>
                                    </div>
                                    <p className="text-xs text-slate-500 truncate">{profile.startUrl}</p>
                                </div>
                            ))
                        )}
                    </div>
                </div>

                {/* Profile Details */}
                <div className="flex-1 flex flex-col overflow-hidden">
                    {selectedProfile ? (
                        <>
                            {/* Profile Header */}
                            <div className="p-4 border-b border-white/5 bg-[#0D1117]">
                                <div className="flex items-center justify-between">
                                    <div>
                                        <h2 className="text-lg font-semibold text-white/90">{selectedProfile.name}</h2>
                                        <p className="text-sm text-slate-400 mt-1">{selectedProfile.startUrl}</p>
                                    </div>
                                    <div className="flex items-center gap-2">
                                        <Button
                                            variant="ghost"
                                            size="sm"
                                            className="text-emerald-400 hover:bg-emerald-500/10"
                                            onClick={() => handleReplayFlow(selectedProfile.id)}
                                        >
                                            <Play size={14} className="mr-1" /> Replay
                                        </Button>
                                        <Button variant="ghost" size="sm" className="text-red-400 hover:bg-red-500/10"
                                            onClick={() => handleDeleteProfile(selectedProfile.id)}
                                        >
                                            <Trash2 size={14} />
                                        </Button>
                                    </div>
                                </div>
                                <div className="flex items-center gap-4 mt-3 text-xs text-slate-500">
                                    <span>Created: {formatDate(selectedProfile.createdAt)}</span>
                                    <span>Updated: {formatDate(selectedProfile.updatedAt)}</span>
                                </div>
                            </div>

                            {/* Executions */}
                            <div className="flex-1 overflow-y-auto p-4">
                                <h3 className="text-sm font-medium text-white/70 mb-3">Execution History</h3>
                                {executionsLoading ? (
                                    <div className="flex items-center justify-center p-8">
                                        <div className="w-5 h-5 border-2 border-cyan-500/30 border-t-cyan-500 rounded-full animate-spin" />
                                    </div>
                                ) : executions.length === 0 ? (
                                    <div className="flex flex-col items-center justify-center p-8 text-slate-500 bg-white/5 rounded-lg">
                                        <Clock size={24} className="mb-2 opacity-50" />
                                        <p className="text-sm">No executions yet</p>
                                        <p className="text-xs mt-1">Click Replay to run this flow</p>
                                    </div>
                                ) : (
                                    <div className="space-y-2">
                                        {executions.map((exec) => (
                                            <div
                                                key={exec.id}
                                                className="p-3 bg-[#161B22] rounded-lg border border-white/5"
                                            >
                                                <div className="flex items-center justify-between mb-2">
                                                    <div className="flex items-center gap-2">
                                                        {exec.status === 'success' ? (
                                                            <CheckCircle size={14} className="text-emerald-400" />
                                                        ) : exec.status === 'failed' ? (
                                                            <XCircle size={14} className="text-red-400" />
                                                        ) : (
                                                            <RefreshCw size={14} className="text-cyan-400 animate-spin" />
                                                        )}
                                                        <span className="text-sm text-white/80">
                                                            {exec.status === 'success' ? 'Completed' : exec.status === 'failed' ? 'Failed' : 'Running'}
                                                        </span>
                                                    </div>
                                                    <span className="text-xs text-slate-500">
                                                        {formatDate(exec.startedAt)}
                                                    </span>
                                                </div>
                                                <div className="flex items-center gap-4 text-xs text-slate-500">
                                                    <span>Steps: {exec.stepsCompleted}/{exec.totalSteps}</span>
                                                    <span>Duration: {formatDuration(exec.durationMs)}</span>
                                                    <span>Agent: {exec.agentId}</span>
                                                </div>
                                                {exec.errorMessage && (
                                                    <div className="mt-2 p-2 bg-red-500/10 rounded text-xs text-red-400 font-mono">
                                                        {exec.errorMessage}
                                                    </div>
                                                )}
                                            </div>
                                        ))}
                                    </div>
                                )}
                            </div>
                        </>
                    ) : (
                        <div className="flex-1 flex items-center justify-center text-slate-500">
                            <div className="text-center">
                                <Video size={48} className="mx-auto mb-4 opacity-30" />
                                <p>Select a flow profile to view details</p>
                            </div>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
};

export default FlowRecorderView;
