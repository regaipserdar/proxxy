import { GoogleGenAI } from "@google/genai";
import { ProxxyNode } from '@/types';
import { Edge } from "reactflow";

// Ensure we have a string even if env is missing
const apiKey = import.meta.env.VITE_GEMINI_API_KEY || "";
const ai = new GoogleGenAI({ apiKey });

export const generateWorkflowCode = async (nodes: ProxxyNode[], edges: Edge[]): Promise<string> => {
  try {
    const prompt = `Generate a Rust-based Proxxy Traffic Policy configuration for the following flow.
    Nodes: ${JSON.stringify(nodes.map(n => n.data))}
    Connections: ${JSON.stringify(edges.map(e => ({ from: e.source, to: e.target })))}
    Output only the configuration JSON or DSL code.`;

    const response = await ai.models.generateContent({
      model: "gemini-1.5-flash",
      contents: prompt,
    });

    return response.text || "// No code generated.";
  } catch (err) {
    console.error("Gemini Error:", err);
    return "// Error generating workflow code. Please check your API key.";
  }
};

export const getDebugInsights = async (logs: string[]): Promise<string> => {
  try {
    const prompt = `Interpret these execution logs and provide a brief technical insight:
    ${logs.join('\n')}`;

    const response = await ai.models.generateContent({
      model: "gemini-1.5-flash",
      contents: prompt,
    });

    return response.text || "No insights available.";
  } catch (err) {
    console.error("Gemini Error:", err);
    return "No insights available.";
  }
};
