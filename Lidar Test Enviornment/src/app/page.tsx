"use client"

import React, { useEffect, useState, useRef, useCallback } from 'react';
import ControlPanel from '@/components/ControlPanel';
import MapCanvas from '@/components/MapCanvas';
import DiagnosticsBar from '@/components/DiagnosticsBar';

/**
 * Type declarations for the Electron IPC bridge
 */
declare global {
  interface Window {
    api?: {
      onMapUpdate: (cb: (data: MapUpdateData) => void) => () => void;
      onLidarStatus: (cb: (status: string) => void) => () => void;
      onConnectedPort: (cb: (port: string) => void) => () => void;
      onAvailablePorts: (cb: (ports: PortInfo[]) => void) => () => void;
      getStatus: () => Promise<string>;
      listPorts: () => Promise<PortInfo[]>;
      connectPort: (port: string) => Promise<boolean>;
      disconnectPort: () => Promise<boolean>;
      updatePipelineConfig: (moduleId: string, config: Record<string, unknown>) => void;
      clearMap: () => void;
      startRecording: () => Promise<boolean>;
      stopRecording: () => Promise<string | null>;
      runCalibration: () => Promise<any>;
      onCalibrationProgress: (cb: (progress: number) => void) => () => void;
      onCalibrationComplete: (cb: (result: any) => void) => () => void;
    };
  }
}

interface PortInfo {
  path: string;
  manufacturer: string;
  vendorId?: string;
  productId?: string;
}

interface WallSegment {
  id: number;
  start: { x: number; y: number };
  end: { x: number; y: number };
  confidence: number;
  length: number;
  angle: number;
  center: { x: number; y: number };
  pointCount: number;
}

interface MovingObject {
  id: number;
  center: { x: number; y: number };
  radius: number;
  pointCount: number;
  angleSpan: number;
  avgDiff: number;
  points: { x: number; y: number }[];
  trail: { x: number; y: number }[];
}

interface GridSnapshot {
  cells: { x: number; y: number; probability: number; logOdds: number }[];
  cellSize: number;
  totalCells: number;
}

interface DiagnosticsData {
  rawCount: number;
  filteredCount: number;
  downsampledCount: number;
  cleanedCount: number;
  wallCount: number;
  gridCellCount: number;
  processTimeMs: number;
  icpConverged: boolean;
  icpIterations: number;
  sweepNumber: number;
  profileSweeps: number;
}

interface MapUpdateData {
  points: { x: number; y: number; angle: number; distance: number }[];
  walls: WallSegment[];
  movingObjects?: MovingObject[];
  grid: GridSnapshot;
  pose: { x: number; y: number; theta: number };
  diagnostics: DiagnosticsData;
}

interface PipelineConfig {
  preFilter: { minRange: number; maxRange: number; minConfidence: number };
  downsampler: { cellSize: number; enabled: boolean };
  outlierRemoval: { enabled: boolean; neighborCount: number; stddevMultiplier: number };
  occupancyGrid: { cellSize: number; hitIncrement: number; missDecrement: number; raycastEnabled: boolean };
  scanMatcher: { enabled: boolean };
  wallExtractor: {
    emaAlpha: number;
    minWallBins: number;
    lineDeviationThreshold: number;
    minWallLength: number;
    cornerAngleThreshold: number;
    enabled: boolean;
  };
  movingObjectDetector: {
    enabled: boolean;
    distanceThreshold: number;
    relativeThreshold: number;
    minObjectPoints: number;
    maxObjectGapDeg: number;
    onlyCloser: boolean;
    cooldownSweeps: number;
  };
  visuals: {
    showGrid: boolean;
    showPoints: boolean;
    showWalls: boolean;
    showMovingObjects: boolean;
    showPose: boolean;
    wallThickness: number;
    wallOpacity: number;
    scanBeam: boolean;
  };
}

const DEFAULT_CONFIG: PipelineConfig = {
  preFilter: { minRange: 150, maxRange: 12000, minConfidence: 100 },
  downsampler: { cellSize: 30, enabled: true },
  outlierRemoval: { enabled: true, neighborCount: 8, stddevMultiplier: 1.5 },
  occupancyGrid: { cellSize: 50, hitIncrement: 0.9, missDecrement: 0.4, raycastEnabled: true },
  scanMatcher: { enabled: false },
  wallExtractor: {
    emaAlpha: 0.15,
    minWallBins: 10,
    lineDeviationThreshold: 40,
    minWallLength: 200,
    cornerAngleThreshold: 25,
    enabled: true,
  },
  movingObjectDetector: {
    enabled: true,
    distanceThreshold: 100,
    relativeThreshold: 0.15,
    minObjectPoints: 3,
    maxObjectGapDeg: 5,
    onlyCloser: false,
    cooldownSweeps: 5,
  },
  visuals: {
    showGrid: true,
    showPoints: true,
    showWalls: true,
    showMovingObjects: true,
    showPose: true,
    wallThickness: 70,
    wallOpacity: 1.0,
    scanBeam: true,
  },
};

export default function Home() {
  const [mapData, setMapData] = useState<MapUpdateData | null>(null);
  const [status, setStatus] = useState('disconnected');
  const [connectedPort, setConnectedPort] = useState<string | null>(null);
  const [availablePorts, setAvailablePorts] = useState<PortInfo[]>([]);
  const [zoom, setZoom] = useState(10000);
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  const [isPaused, setIsPaused] = useState(false);
  const [isRecording, setIsRecording] = useState(false);
  const [config, setConfig] = useState<PipelineConfig>(DEFAULT_CONFIG);
  const [isCalibrating, setIsCalibrating] = useState(false);
  const [calibrationProgress, setCalibrationProgress] = useState(0);
  const isPausedRef = useRef(false);
  useEffect(() => { isPausedRef.current = isPaused; }, [isPaused]);

  // Push config changes to the pipeline
  const updateModuleConfig = useCallback((moduleId: string, moduleConfig: Record<string, unknown>) => {
    setConfig(prev => ({ ...prev, [moduleId]: { ...prev[moduleId as keyof PipelineConfig] as object, ...moduleConfig } }));
    if (window.api && moduleId !== 'visuals') {
      window.api.updatePipelineConfig(moduleId, moduleConfig);
    }
  }, []);

  // Data and status listeners
  useEffect(() => {
    if (!window.api) return;

    window.api.getStatus().then(s => setStatus(s));
    window.api.listPorts().then(ports => setAvailablePorts(ports));

    const cleanupData = window.api.onMapUpdate((data: MapUpdateData) => {
      if (isPausedRef.current) return;
      setMapData(data);
    });

    const cleanupStatus = window.api.onLidarStatus((s: string) => setStatus(s));
    const cleanupPort = window.api.onConnectedPort((port: string) => setConnectedPort(port));
    const cleanupPorts = window.api.onAvailablePorts((ports: PortInfo[]) => setAvailablePorts(ports));
    const cleanupCalProg = window.api.onCalibrationProgress((p: number) => setCalibrationProgress(p));
    const cleanupCalDone = window.api.onCalibrationComplete((result: any) => {
      setIsCalibrating(false);
      setCalibrationProgress(0);
      if (result.bestConfig) {
        setConfig(prev => ({ ...prev, wallExtractor: { ...prev.wallExtractor, ...result.bestConfig } }));
      }
    });

    return () => {
      cleanupData();
      cleanupStatus();
      cleanupPort();
      cleanupPorts();
      cleanupCalProg();
      cleanupCalDone();
    };
  }, []);

  const handleRunCalibration = useCallback(async () => {
    if (!window.api || isCalibrating) return;
    setIsCalibrating(true);
    setCalibrationProgress(0);
    const result = await window.api.runCalibration();
    if (result.error) {
       console.error("Calibration failed:", result.error);
       setIsCalibrating(false);
    }
  }, [isCalibrating]);

  // Keyboard shortcuts
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.code === 'Space') { e.preventDefault(); setIsPaused(p => !p); }
      if (e.code === 'Home') { setOffset({ x: 0, y: 0 }); setZoom(10000); }
      if (e.code === 'KeyR' && e.ctrlKey) {
        e.preventDefault();
        if (!isRecording) {
          window.api?.startRecording();
          setIsRecording(true);
        } else {
          window.api?.stopRecording();
          setIsRecording(false);
        }
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [isRecording]);

  const handleConnect = useCallback(async (portPath: string) => {
    if (window.api) await window.api.connectPort(portPath);
  }, []);

  const handleDisconnect = useCallback(async () => {
    if (window.api) await window.api.disconnectPort();
    setConnectedPort(null);
  }, []);

  const handleRefreshPorts = useCallback(async () => {
    if (window.api) {
      const ports = await window.api.listPorts();
      setAvailablePorts(ports);
    }
  }, []);

  const handleClearMap = useCallback(() => {
    if (window.api) window.api.clearMap();
    setMapData(null);
  }, []);

  const handleRecenter = useCallback(() => {
    setOffset({ x: 0, y: 0 });
    setZoom(10000);
  }, []);

  return (
    <div className="flex h-screen w-screen overflow-hidden" style={{ background: 'var(--bg-deep)' }}>
      {/* Left: Control Panel */}
      <ControlPanel
        status={status}
        connectedPort={connectedPort}
        availablePorts={availablePorts}
        config={config}
        onUpdateConfig={updateModuleConfig}
        onConnect={handleConnect}
        onDisconnect={handleDisconnect}
        onRefreshPorts={handleRefreshPorts}
        onClearMap={handleClearMap}
        onRecenter={handleRecenter}
        isPaused={isPaused}
        setIsPaused={setIsPaused}
        isRecording={isRecording}
        setIsRecording={setIsRecording}
        onRunCalibration={handleRunCalibration}
        isCalibrating={isCalibrating}
        calibrationProgress={calibrationProgress}
      />

      {/* Center + Bottom: Map and Diagnostics */}
      <div className="flex-1 flex flex-col min-w-0">
        {/* Map Header */}
        <header className="flex items-center justify-between px-8 py-4 flex-shrink-0" style={{ borderBottom: '1px solid var(--border-dim)' }}>
          <div>
            <h1 className="text-2xl font-black tracking-tight" style={{ color: 'var(--text-primary)' }}>
              SLAM <span style={{ color: 'var(--accent-cyan)' }}>Map</span>
            </h1>
            <p className="text-[10px] font-bold uppercase tracking-[0.3em]" style={{ color: 'var(--text-dim)' }}>
              Probabilistic Occupancy Grid • RANSAC Extraction • ICP Alignment
            </p>
          </div>

          <div className="flex items-center gap-4">
            {/* Zoom Control */}
            <div className="flex items-center gap-3 px-5 py-2 rounded-xl" style={{ background: 'var(--bg-panel)', border: '1px solid var(--border-dim)' }}>
              <div className="flex flex-col">
                <span className="text-[9px] font-bold uppercase tracking-widest" style={{ color: 'var(--text-dim)' }}>Range</span>
                <span className="text-xs font-black" style={{ color: 'var(--accent-cyan)' }}>{Math.round(zoom)}mm</span>
              </div>
              <input
                type="range" min="1000" max="25000" step="500"
                value={zoom}
                onChange={(e) => setZoom(parseInt(e.target.value))}
                className="w-40 h-1"
              />
            </div>

            {/* Status Badges */}
            {isPaused && (
              <div className="flex items-center gap-2 px-4 py-1.5 rounded-full text-[10px] font-black uppercase tracking-widest"
                style={{ background: 'rgba(245, 158, 11, 0.15)', color: 'var(--accent-amber)', border: '1px solid rgba(245, 158, 11, 0.3)' }}>
                <div className="w-1.5 h-1.5 rounded-full animate-pulse-glow" style={{ background: 'var(--accent-amber)' }} />
                Paused
              </div>
            )}
            {isRecording && (
              <div className="flex items-center gap-2 px-4 py-1.5 rounded-full text-[10px] font-black uppercase tracking-widest"
                style={{ background: 'rgba(239, 68, 68, 0.15)', color: 'var(--accent-red)', border: '1px solid rgba(239, 68, 68, 0.3)' }}>
                <div className="w-1.5 h-1.5 rounded-full animate-pulse-glow" style={{ background: 'var(--accent-red)' }} />
                Recording
              </div>
            )}
          </div>
        </header>

        {/* Map Canvas */}
        <section className="flex-1 relative min-h-0">
          <MapCanvas
            mapData={mapData}
            zoom={zoom}
            offset={offset}
            setOffset={setOffset}
            setZoom={setZoom}
            config={config.visuals}
            isPaused={isPaused}
          />
        </section>

        {/* Diagnostics Bar */}
        <DiagnosticsBar
          status={status}
          diagnostics={mapData?.diagnostics || null}
          pose={mapData?.pose || null}
        />
      </div>
    </div>
  );
}
