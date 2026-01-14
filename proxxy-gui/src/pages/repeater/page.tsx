import { useState, useEffect, useMemo } from 'react';
import { useQuery, useMutation } from '@apollo/client';
import { useRepeaterStore } from '@/store/repeaterStore';
import { GET_AGENTS, EXECUTE_REPEATER_REQUEST } from '@/graphql/operations';
import { parseRawRequest } from '@/lib/http-utils';

// Components
import { RepeaterTabs } from '@/components/repeater/RepeaterTabs';
import { RepeaterToolbar } from '@/components/repeater/RepeaterToolbar';
import { RepeaterInspector } from '@/components/repeater/RepeaterInspector';
import { EmptyRepeater } from '@/components/repeater/EmptyRepeater';
import { RepeaterAgent } from '@/components/repeater/types';

export const RepeaterView = () => {
  const { tasks, activeTaskId, addTask, updateTask, removeTask, setActiveTaskId } = useRepeaterStore();

  const activeTask = useMemo(() =>
    tasks.find((t: any) => t.id === activeTaskId) || tasks[0],
    [tasks, activeTaskId]);

  // UI State
  const [isSending, setIsSending] = useState(false);
  const [reqSearch, setReqSearch] = useState('');
  const [resSearch, setResSearch] = useState('');
  const [isEditingName, setIsEditingName] = useState(false);
  const [editingNameValue, setEditingNameValue] = useState('');
  const [isAgentMenuOpen, setIsAgentMenuOpen] = useState(false);

  // Agent Selection State
  const { data: agentsData } = useQuery(GET_AGENTS, {
    pollInterval: 5000
  });

  const agents = useMemo<RepeaterAgent[]>(() => {
    return (agentsData?.agents || []).map((a: any) => ({
      ...a,
      type: a.id === 'local' ? 'local' : 'cloud',
      version: a.version || 'unknown'
    }));
  }, [agentsData]);

  const [selectedAgentId, setSelectedAgentId] = useState<string | null>(null);

  const selectedAgent = useMemo(() =>
    agents.find((a: any) => a.id === selectedAgentId) || agents[0],
    [agents, selectedAgentId]);

  useEffect(() => {
    if (agents.length > 0 && !selectedAgentId) {
      setSelectedAgentId(agents[0].id);
    }
  }, [agents, selectedAgentId]);

  const [executeRequest] = useMutation(EXECUTE_REPEATER_REQUEST);

  const handleSend = async () => {
    if (!activeTask || !selectedAgent) return;
    if (selectedAgent.status.toLowerCase() !== 'online') {
      alert("Bu agent şu an offline!");
      return;
    }

    setIsSending(true);

    try {
      const parsed = parseRawRequest(activeTask.request);
      if (!parsed) {
        alert("Invalid request format!");
        setIsSending(false);
        return;
      }

      const { data } = await executeRequest({
        variables: {
          input: {
            tabId: activeTask.id,
            targetAgentId: selectedAgent.id,
            requestData: {
              method: parsed.method,
              url: parsed.url,
              headers: parsed.headers,
              body: parsed.body
            }
          }
        }
      });

      if (data?.executeRepeaterRequest?.responseData) {
        const res = data.executeRepeaterRequest.responseData;
        // Construct raw response string
        let rawResponse = `HTTP/1.1 ${res.statusCode}\n`;
        if (res.headers) {
          try {
            const headers = JSON.parse(res.headers);
            Object.entries(headers).forEach(([k, v]) => {
              rawResponse += `${k}: ${v}\n`;
            });
          } catch (e) {
            console.error("Failed to parse response headers", e);
          }
        }
        rawResponse += `\n${res.body}`;

        updateTask(activeTask.id, { response: rawResponse });
      } else if (data?.executeRepeaterRequest?.error) {
        updateTask(activeTask.id, { response: `ERROR: ${data.executeRepeaterRequest.error}` });
      }
    } catch (err: any) {
      console.error("Execution failed:", err);
      updateTask(activeTask.id, { response: `CLIENT_ERROR: ${err.message}` });
    } finally {
      setIsSending(false);
    }
  };

  const handleNewTab = () => {
    addTask({
      name: 'New Request',
      request: `GET / HTTP/1.1\nHost: example.com\n\n`
    });
  };

  const startEditingName = () => {
    if (activeTask) {
      setEditingNameValue(activeTask.name);
      setIsEditingName(true);
    }
  };

  const saveName = () => {
    if (activeTask && editingNameValue.trim()) {
      updateTask(activeTask.id, { name: editingNameValue });
    }
    setIsEditingName(false);
  };

  if (tasks.length === 0) {
    return <EmptyRepeater handleNewTab={handleNewTab} />;
  }

  return (
    <div className="flex flex-col h-full bg-[#0A0E14] text-white/80 font-sans selection:bg-[#9DCDE8]/30 overflow-hidden">
      <RepeaterTabs
        tasks={tasks}
        activeTaskId={activeTaskId}
        setActiveTaskId={setActiveTaskId}
        removeTask={removeTask}
        handleNewTab={handleNewTab}
        startEditingName={startEditingName}
      />

      <RepeaterToolbar
        activeTask={activeTask}
        isEditingName={isEditingName}
        editingNameValue={editingNameValue}
        setEditingNameValue={setEditingNameValue}
        saveName={saveName}
        startEditingName={startEditingName}
        agents={agents}
        selectedAgentId={selectedAgentId}
        setSelectedAgentId={setSelectedAgentId}
        isAgentMenuOpen={isAgentMenuOpen}
        setIsAgentMenuOpen={setIsAgentMenuOpen}
        handleSend={handleSend}
        isSending={isSending}
      />

      <RepeaterInspector
        activeTask={activeTask}
        updateTask={updateTask}
        reqSearch={reqSearch}
        setReqSearch={setReqSearch}
        resSearch={resSearch}
        setResSearch={setResSearch}
      />
    </div>
  );
};