import React, { useEffect } from 'react';
import { useQuery } from '@apollo/client';
import { GET_PROJECTS } from '../graphql/operations';
import { useNavigate } from 'react-router-dom';
import { Loader2 } from 'lucide-react';
import { OrchestratorConnectionError } from './OrchestratorConnectionError';

export function ProjectGuard({ children }: { children: React.ReactNode }) {
    const navigate = useNavigate();

    // We poll frequently to detect if project was unloaded or switched
    const { data, loading, error, refetch } = useQuery(GET_PROJECTS, {
        pollInterval: 5000,
        fetchPolicy: 'network-only' // Ensure we get fresh status
    });



    // Determine if any project is active
    const activeProject = data?.projects?.find((p: any) => p.isActive);

    useEffect(() => {
        if (!loading && !activeProject) {
            // No active project, redirect to launcher
            // But only if we are not already there (though guard shouldn't wrap launcher)
            navigate('/projects');
        }
    }, [loading, activeProject, navigate]);

    if (loading && !data) {
        return (
            <div className="h-screen w-full bg-slate-950 flex items-center justify-center">
                <Loader2 className="w-8 h-8 text-emerald-500 animate-spin" />
            </div>
        );
    }

    if (error) {
        return <OrchestratorConnectionError onRetry={() => refetch()} />;
    }

    if (!activeProject) {
        return null; // Will redirect via useEffect
    }

    return <>{children}</>;
}
