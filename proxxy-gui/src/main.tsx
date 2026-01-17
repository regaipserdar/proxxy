import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ApolloProvider } from '@apollo/client/react';
import { apolloClient } from './graphql';
import { Toaster } from 'sonner';
import { loader } from '@monaco-editor/react';
import * as monaco from 'monaco-editor';

// Configure Monaco to use local files instead of CDN
loader.config({ monaco });

const queryClient = new QueryClient();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ApolloProvider client={apolloClient}>
      <QueryClientProvider client={queryClient}>
        <App />
        <Toaster position="bottom-right" richColors theme="dark" />
      </QueryClientProvider>
    </ApolloProvider>
  </React.StrictMode>,
);
