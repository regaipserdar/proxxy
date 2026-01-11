import React, { useMemo, useState, useEffect } from 'react';
import { Activity, ArrowUpRight } from 'lucide-react';
import { HttpTransaction } from '../../types/graphql';
import { Card, CardContent } from "@/components/ui/card";

interface TrafficSummaryCardProps {
  traffic: HttpTransaction[];
}

// Sparkline grafiği için maksimum veri noktası sayısı
const HISTORY_LENGTH = 30;

export const TrafficSummaryCard: React.FC<TrafficSummaryCardProps> = ({ traffic }) => {
  const [rps, setRps] = useState(0);
  const [peakRps, setPeakRps] = useState(0);
  // Grafik geçmişi için state [0, 5, 12, 4, 0...]
  const [history, setHistory] = useState<number[]>(new Array(HISTORY_LENGTH).fill(0));

  // İstatistikleri hesapla
  const stats = useMemo(() => {
    const total = traffic.length;
    if (total === 0) return { statusDist: { '2xx': 0, '3xx': 0, '4xx': 0, '5xx': 0 }, total: 0 };

    const dist = { '2xx': 0, '3xx': 0, '4xx': 0, '5xx': 0 };

    traffic.forEach(tx => {
      const s = tx.status || 0;
      if (s >= 200 && s < 300) dist['2xx']++;
      else if (s >= 300 && s < 400) dist['3xx']++;
      else if (s >= 400 && s < 500) dist['4xx']++;
      else if (s >= 500) dist['5xx']++;
    });

    return { statusDist: dist, total };
  }, [traffic]);

  // RPS Hesaplama ve Grafik Güncelleme Döngüsü
  useEffect(() => {
    const interval = setInterval(() => {
      const now = Date.now();
      // Son 1 saniyedeki istekleri say (Timestamp kontrolü)
      // Not: Backend timestamp saniye ise *1000 yapıyoruz, ms ise direkt kullanıyoruz
      const recentCount = traffic.filter(tx => {
        const ts = Number(tx.timestamp);
        const timeMs = ts > 1e11 ? ts : ts * 1000;
        return (now - timeMs) < 1000;
      }).length;

      setRps(recentCount);
      if (recentCount > peakRps) setPeakRps(recentCount);

      // Geçmiş dizisini kaydır: [a, b, c] -> [b, c, new]
      setHistory(prev => {
        const newHistory = [...prev.slice(1), recentCount];
        return newHistory;
      });

    }, 1000); // Her saniye güncelle

    return () => clearInterval(interval);
  }, [traffic, peakRps]);

  // SVG Path Oluşturucu (Basit Sparkline)
  const getPath = () => {
    const max = Math.max(...history, 10); // Ölçekleme için max değer (min 10)
    const width = 100; // SVG viewBox genişliği %
    const height = 40; // SVG viewBox yüksekliği
    const step = width / (HISTORY_LENGTH - 1);

    const points = history.map((val, i) => {
      const x = i * step;
      // Y eksenini ters çevir (SVG'de 0 yukarıdadır)
      const y = height - (val / max) * height;
      return `${x},${y}`;
    });

    return `M ${points.join(' L ')}`;
  };

  const isLive = rps > 0;

  return (
    <Card className="bg-[#111318] border-white/5 shadow-xl h-full relative overflow-hidden group">
      {/* Üst Çizgi Göstergesi */}
      <div className={`absolute top-0 left-0 w-full h-0.5 ${isLive ? 'bg-indigo-500 shadow-[0_0_10px_#6366f1]' : 'bg-slate-700'} transition-all duration-500`} />

      <CardContent className="p-5 flex flex-col h-full justify-between">

        {/* Header */}
        <div className="flex justify-between items-start mb-2">
          <div>
            <div className="flex items-center gap-2 mb-1">
              <Activity className={`w-4 h-4 ${isLive ? 'text-indigo-400' : 'text-slate-500'}`} />
              <span className="text-xs font-bold text-slate-400 uppercase tracking-wider">Network I/O</span>
            </div>
            <div className="flex items-baseline gap-2">
              <span className={`text-3xl font-mono font-bold tracking-tight transition-colors duration-300 ${isLive ? 'text-white' : 'text-slate-500'}`}>
                {rps.toFixed(1)}
              </span>
              <span className="text-xs text-slate-500 font-medium">req/sec</span>
            </div>
          </div>

          <div className="text-right">
            <div className="flex items-center gap-1 justify-end text-[10px] font-bold text-slate-500 uppercase tracking-wider">
              Peak
              <ArrowUpRight className="w-3 h-3" />
            </div>
            <div className="text-sm font-mono font-bold text-indigo-400">
              {peakRps.toFixed(0)}
            </div>
          </div>
        </div>

        {/* Sparkline Chart Area */}
        <div className="h-12 w-full mb-4 relative">
          <svg className="w-full h-full overflow-visible" viewBox="0 0 100 40" preserveAspectRatio="none">
            {/* Gradient Definition */}
            <defs>
              <linearGradient id="gradient" x1="0" x2="0" y1="0" y2="1">
                <stop offset="0%" stopColor="#6366f1" stopOpacity="0.5" />
                <stop offset="100%" stopColor="#6366f1" stopOpacity="0" />
              </linearGradient>
            </defs>

            {/* Area Fill */}
            <path
              d={`${getPath()} L 100,40 L 0,40 Z`}
              fill="url(#gradient)"
              className="transition-all duration-300 ease-linear"
            />

            {/* Stroke Line */}
            <path
              d={getPath()}
              fill="none"
              stroke={isLive ? '#818cf8' : '#334155'}
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
              className="transition-all duration-300 ease-linear vector-effect-non-scaling-stroke"
            />
          </svg>
        </div>

        {/* Status Distribution Bar */}
        <div className="mt-auto pt-2 border-t border-white/5 space-y-2">
          <div className="flex justify-between items-center text-[10px] font-medium text-slate-500">
            <span>Response Codes</span>
            <span>{stats.total} Total</span>
          </div>

          {/* Multi-colored Progress Bar */}
          <div className="h-2 w-full bg-white/5 rounded-full overflow-hidden flex">
            {/* 2xx - Green */}
            <div
              className="h-full bg-emerald-500 transition-all duration-500"
              style={{ width: `${(stats.statusDist['2xx'] / stats.total) * 100 || 0}%` }}
            />
            {/* 3xx - Blue */}
            <div
              className="h-full bg-blue-500 transition-all duration-500"
              style={{ width: `${(stats.statusDist['3xx'] / stats.total) * 100 || 0}%` }}
            />
            {/* 4xx - Yellow */}
            <div
              className="h-full bg-amber-500 transition-all duration-500"
              style={{ width: `${(stats.statusDist['4xx'] / stats.total) * 100 || 0}%` }}
            />
            {/* 5xx - Red */}
            <div
              className="h-full bg-red-500 transition-all duration-500"
              style={{ width: `${(stats.statusDist['5xx'] / stats.total) * 100 || 0}%` }}
            />
          </div>

          {/* Legend */}
          <div className="flex justify-between gap-1 text-[9px] font-mono text-slate-500">
            <span className="flex items-center gap-1"><span className="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>2xx</span>
            <span className="flex items-center gap-1"><span className="w-1.5 h-1.5 rounded-full bg-blue-500"></span>3xx</span>
            <span className="flex items-center gap-1"><span className="w-1.5 h-1.5 rounded-full bg-amber-500"></span>4xx</span>
            <span className="flex items-center gap-1"><span className="w-1.5 h-1.5 rounded-full bg-red-500"></span>5xx</span>
          </div>
        </div>

      </CardContent>
    </Card>
  );
};