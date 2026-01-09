import React from 'react';
import { AlertTriangle, RefreshCw } from 'lucide-react';

interface ErrorMessageProps {
  title: string;
  message: string;
  onRetry?: () => void;
  className?: string;
}

export const ErrorMessage: React.FC<ErrorMessageProps> = ({ 
  title, 
  message, 
  onRetry, 
  className = '' 
}) => {
  return (
    <div className={`bg-red-500/10 border border-red-500/20 rounded-xl p-6 ${className}`}>
      <div className="flex items-start gap-4">
        <div className="p-2 rounded-lg bg-red-500/20">
          <AlertTriangle className="h-5 w-5 text-red-400" />
        </div>
        <div className="flex-1">
          <h3 className="text-lg font-bold text-red-400 mb-2">{title}</h3>
          <p className="text-white/80 text-sm mb-4">{message}</p>
          {onRetry && (
            <button
              onClick={onRetry}
              className="flex items-center gap-2 px-4 py-2 bg-red-500/20 hover:bg-red-500/30 border border-red-500/30 rounded-lg text-red-400 text-sm font-bold transition-all"
            >
              <RefreshCw className="h-4 w-4" />
              Retry
            </button>
          )}
        </div>
      </div>
    </div>
  );
};