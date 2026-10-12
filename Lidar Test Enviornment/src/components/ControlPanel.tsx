"use client"

import React, { useState } from 'react';

interface ControlPanelProps {
  status: string;
  connectedPort: string | null;
  availablePorts: { path: string; manufacturer: string }[];
  config: any;
  onUpdateConfig: (moduleId: string, config: Record<string, unknown>) => void;
  onConnect: (port: string) => void;
  onDisconnect: () => void;
  onRefreshPorts: () => void;
  onClearMap: () => void;
  onRecenter: () => void;
  isPaused: boolean;
  setIsPaused: (p: boolean) => void;
  isRecording: boolean;
  setIsRecording: (r: boolean) => void;
  onRunCalibration: () => void;
  isCalibrating: boolean;
  calibrationProgress: number;
}

// Reusable slider component
function SliderControl({ label, value, min, max, step, unit, color, onChange }: {
  label: string; value: number; min: number; max: number; step: number;
  unit: string; color?: string; onChange: (v: number) => void;
}) {
  return (
    <div className="space-y-2">
      <div className="flex justify-between items-center">
        <span className="text-[11px] font-semibold" style={{ color: 'var(--text-secondary)' }}>{label}</span>
        <span className="text-[10px] font-bold px-2 py-0.5 rounded-md"
          style={{ color: color || 'var(--accent-cyan)', background: color ? `${color}15` : 'var(--accent-cyan-dim)' }}>
          {value}{unit}
        </span>
      </div>
      <input
        type="range" min={min} max={max} step={step}
        value={value}
        onChange={(e) => onChange(parseFloat(e.target.value))}
        className={`w-full ${color === 'var(--accent-amber)' ? 'accent-amber' : ''}`}
      />
    </div>
  );
}

// Reusable toggle component
function ToggleControl({ label, subtitle, checked, onChange }: {
  label: string; subtitle?: string; checked: boolean; onChange: (v: boolean) => void;
}) {
  return (
    <label className="flex items-center justify-between cursor-pointer group py-1">
      <div className="flex flex-col">
        <span className="text-[11px] font-semibold group-hover:text-cyan-400 transition-colors" style={{ color: 'var(--text-secondary)' }}>
          {label}
        </span>
        {subtitle && (
          <span className="text-[8px] font-semibold uppercase tracking-wider" style={{ color: 'var(--text-dim)' }}>
            {subtitle}
          </span>
        )}
      </div>
      <div
        className={`toggle-track ${checked ? 'active' : ''}`}
        onClick={(e) => { e.preventDefault(); onChange(!checked); }}
      >
        <div className="toggle-thumb" />
      </div>
    </label>
  );
}

// Collapsible Section
function Section({ title, badge, children, defaultOpen = true }: {
  title: string; badge?: string; children: React.ReactNode; defaultOpen?: boolean;
}) {
  const [isOpen, setIsOpen] = useState(defaultOpen);

  return (
    <div className="space-y-3">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="flex items-center justify-between w-full group"
      >
        <div className="flex items-center gap-2">
          <span className="text-[9px] uppercase tracking-[0.2em] font-black" style={{ color: 'var(--text-dim)' }}>
            {title}
          </span>
          {badge && (
            <span className="text-[7px] font-bold px-1.5 py-0.5 rounded" style={{ background: 'var(--accent-cyan-dim)', color: 'var(--accent-cyan)' }}>
              {badge}
            </span>
          )}
        </div>
        <svg className={`w-3 h-3 transition-transform ${isOpen ? 'rotate-180' : ''}`} fill="none" stroke="currentColor" viewBox="0 0 24 24" style={{ color: 'var(--text-dim)' }}>
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M19 9l-7 7-7-7" />
        </svg>
      </button>
      {isOpen && (
        <div className="space-y-3 animate-fade-in">
          {children}
        </div>
      )}
    </div>
  );
}

export default function ControlPanel({
  status, connectedPort, availablePorts, config, onUpdateConfig,
  onConnect, onDisconnect, onRefreshPorts, onClearMap, onRecenter,
  isPaused, setIsPaused, isRecording, setIsRecording,
  onRunCalibration, isCalibrating, calibrationProgress
}: ControlPanelProps) {

  const statusConfig: Record<string, { color: string; bg: string; label: string }> = {
    connected: { color: 'var(--accent-green)', bg: 'rgba(16, 185, 129, 0.1)', label: 'Connected' },
    connecting: { color: 'var(--accent-cyan)', bg: 'rgba(34, 211, 238, 0.1)', label: 'Connecting' },
    searching: { color: 'var(--accent-blue)', bg: 'rgba(59, 130, 246, 0.1)', label: 'Searching' },
    error: { color: 'var(--accent-red)', bg: 'rgba(239, 68, 68, 0.1)', label: 'Error' },
    disconnected: { color: 'var(--text-dim)', bg: 'rgba(100, 116, 139, 0.1)', label: 'Disconnected' },
  };

  const currentStatus = statusConfig[status] || statusConfig.disconnected;

  const handleRecordToggle = () => {
    if (!isRecording) {
      window.api?.startRecording();
      setIsRecording(true);
    } else {
      window.api?.stopRecording();
      setIsRecording(false);
    }
  };

  return (
    <aside
      className="w-[310px] flex-shrink-0 h-full flex flex-col overflow-hidden"
      style={{ background: 'var(--bg-panel)', borderRight: '1px solid var(--border-dim)' }}
    >
      {/* Header */}
      <div className="flex items-center gap-3 px-6 py-5" style={{ borderBottom: '1px solid var(--border-dim)' }}>
        <div className="w-10 h-10 rounded-xl flex items-center justify-center shadow-lg"
          style={{ background: 'linear-gradient(135deg, #22d3ee, #3b82f6)', boxShadow: '0 4px 20px rgba(34, 211, 238, 0.25)' }}>
          <svg className="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-1.447-.894L15 7m0 10V7" />
          </svg>
        </div>
        <div>
          <h2 className="text-lg font-black tracking-tight" style={{ color: 'var(--text-primary)' }}>
            Sennep<span style={{ color: 'var(--accent-cyan)' }}>V2</span>
          </h2>
          <p className="text-[8px] font-bold uppercase tracking-[0.15em]" style={{ color: 'var(--accent-cyan)' }}>SLAM Suite</p>
        </div>
      </div>

      {/* Action Buttons */}
      <div className="grid grid-cols-3 gap-2 px-5 py-4" style={{ borderBottom: '1px solid var(--border-dim)' }}>
        <button onClick={() => setIsPaused(!isPaused)}
          className="flex flex-col items-center gap-1.5 py-3 rounded-xl transition-all active:scale-95"
          style={{
            background: isPaused ? 'rgba(245, 158, 11, 0.1)' : 'var(--bg-elevated)',
            border: `1px solid ${isPaused ? 'rgba(245, 158, 11, 0.3)' : 'var(--border-dim)'}`,
            color: isPaused ? 'var(--accent-amber)' : 'var(--text-secondary)',
          }}>
          <div className={`w-1.5 h-1.5 rounded-full ${isPaused ? 'animate-pulse-glow' : ''}`}
            style={{ background: isPaused ? 'var(--accent-amber)' : 'var(--text-dim)' }} />
          <span className="text-[8px] font-black uppercase tracking-widest">{isPaused ? 'Resume' : 'Pause'}</span>
        </button>

        <button onClick={onRecenter}
          className="flex flex-col items-center gap-1.5 py-3 rounded-xl transition-all active:scale-95"
          style={{ background: 'var(--bg-elevated)', border: '1px solid var(--border-dim)', color: 'var(--text-secondary)' }}>
          <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
          </svg>
          <span className="text-[8px] font-black uppercase tracking-widest">Center</span>
        </button>

        <button onClick={handleRecordToggle}
          className="flex flex-col items-center gap-1.5 py-3 rounded-xl transition-all active:scale-95"
          style={{
            background: isRecording ? 'rgba(239, 68, 68, 0.1)' : 'var(--bg-elevated)',
            border: `1px solid ${isRecording ? 'rgba(239, 68, 68, 0.3)' : 'var(--border-dim)'}`,
            color: isRecording ? 'var(--accent-red)' : 'var(--text-secondary)',
          }}>
          <div className={`w-2 h-2 rounded-full ${isRecording ? 'animate-pulse-glow' : ''}`}
            style={{ background: isRecording ? 'var(--accent-red)' : 'var(--text-dim)' }} />
          <span className="text-[8px] font-black uppercase tracking-widest">{isRecording ? 'Stop' : 'Record'}</span>
        </button>
      </div>

      {/* Scrollable Settings */}
      <div className="flex-1 overflow-y-auto px-5 py-4 space-y-5">

        {/* Connection Section */}
        <Section title="Connection" badge={status === 'connected' ? 'LIVE' : undefined}>
          <div className="flex items-center justify-between py-2 px-3 rounded-lg" style={{ background: currentStatus.bg }}>
            <div className="flex items-center gap-2">
              <div className={`w-2 h-2 rounded-full ${status === 'connected' ? 'animate-pulse-glow' : ''}`} style={{ background: currentStatus.color }} />
              <span className="text-[10px] font-bold uppercase tracking-wider" style={{ color: currentStatus.color }}>{currentStatus.label}</span>
            </div>
            {connectedPort && (
              <span className="text-[9px] font-mono font-bold" style={{ color: 'var(--text-dim)' }}>{connectedPort}</span>
            )}
          </div>

          {/* Port Selector */}
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <select
                className="flex-1 text-[11px] font-semibold py-2 px-3 rounded-lg appearance-none cursor-pointer"
                style={{ background: 'var(--bg-elevated)', border: '1px solid var(--border-dim)', color: 'var(--text-secondary)' }}
                onChange={(e) => { if (e.target.value) onConnect(e.target.value); }}
                value={connectedPort || ''}
              >
                <option value="">Select port...</option>
                {availablePorts.map(p => (
                  <option key={p.path} value={p.path}>{p.path} — {p.manufacturer}</option>
                ))}
              </select>
              <button onClick={onRefreshPorts}
                className="p-2 rounded-lg transition-colors"
                style={{ background: 'var(--bg-elevated)', border: '1px solid var(--border-dim)' }}>
                <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" style={{ color: 'var(--text-dim)' }}>
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                </svg>
              </button>
            </div>
            {connectedPort && (
              <button onClick={onDisconnect}
                className="w-full text-[9px] font-bold uppercase tracking-widest py-2 rounded-lg transition-all"
                style={{ background: 'rgba(239, 68, 68, 0.08)', border: '1px solid rgba(239, 68, 68, 0.2)', color: 'var(--accent-red)' }}>
                Disconnect
              </button>
            )}
          </div>
        </Section>

        <div style={{ borderTop: '1px solid var(--border-dim)' }} />

        {/* Pre-Filter */}
        <Section title="Pre-Filter" badge="PHASE 1">
          <SliderControl label="Min Range (Dead Zone)" value={config.preFilter.minRange} min={0} max={1000} step={50} unit="mm"
            color="var(--accent-red)"
            onChange={(v) => onUpdateConfig('preFilter', { minRange: v })} />
          <SliderControl label="Max Range" value={config.preFilter.maxRange} min={4000} max={16000} step={500} unit="mm"
            onChange={(v) => onUpdateConfig('preFilter', { maxRange: v })} />
          <SliderControl label="Min Confidence" value={config.preFilter.minConfidence} min={0} max={255} step={5} unit=""
            onChange={(v) => onUpdateConfig('preFilter', { minConfidence: v })} />
        </Section>

        <div style={{ borderTop: '1px solid var(--border-dim)' }} />

        {/* Occupancy Grid */}
        <Section title="Occupancy Grid" badge="BAYESIAN" defaultOpen={false}>
          <SliderControl label="Cell Size" value={config.occupancyGrid.cellSize} min={10} max={200} step={10} unit="mm"
            onChange={(v) => onUpdateConfig('occupancyGrid', { cellSize: v })} />
          <SliderControl label="Hit Weight" value={config.occupancyGrid.hitIncrement} min={0.1} max={2.0} step={0.1} unit=""
            onChange={(v) => onUpdateConfig('occupancyGrid', { hitIncrement: v })} />
          <SliderControl label="Miss Weight" value={config.occupancyGrid.missDecrement} min={0.1} max={2.0} step={0.1} unit=""
            onChange={(v) => onUpdateConfig('occupancyGrid', { missDecrement: v })} />
          <ToggleControl label="Raycasting" subtitle="Clear beam path" checked={config.occupancyGrid.raycastEnabled}
            onChange={(v) => onUpdateConfig('occupancyGrid', { raycastEnabled: v })} />
        </Section>

        <div style={{ borderTop: '1px solid var(--border-dim)' }} />

        {/* Scan Matching */}
        <Section title="Scan Matching" badge="ICP" defaultOpen={false}>
          <ToggleControl label="ICP Enabled" subtitle="Frame-to-frame alignment" checked={config.scanMatcher.enabled}
            onChange={(v) => onUpdateConfig('scanMatcher', { enabled: v })} />
        </Section>

        <div style={{ borderTop: '1px solid var(--border-dim)' }} />

        {/* Wall Extraction */}
        <Section title="Wall Extraction" badge="PROFILE">
          <ToggleControl label="Enabled" subtitle="Accumulated polar profile" checked={config.wallExtractor.enabled}
            onChange={(v) => onUpdateConfig('wallExtractor', { enabled: v })} />
          <SliderControl label="Smoothing (EMA α)" value={config.wallExtractor.emaAlpha} min={0.05} max={0.9} step={0.05} unit=""
            color="var(--accent-amber)"
            onChange={(v) => onUpdateConfig('wallExtractor', { emaAlpha: v })} />
          <SliderControl label="Line Deviation" value={config.wallExtractor.lineDeviationThreshold} min={5} max={100} step={5} unit="mm"
            onChange={(v) => onUpdateConfig('wallExtractor', { lineDeviationThreshold: v })} />
          <SliderControl label="Min Wall Length" value={config.wallExtractor.minWallLength} min={50} max={1000} step={25} unit="mm"
            onChange={(v) => onUpdateConfig('wallExtractor', { minWallLength: v })} />
          <SliderControl label="Corner Sensitivity" value={config.wallExtractor.cornerAngleThreshold} min={5} max={45} step={1} unit="°"
            onChange={(v) => onUpdateConfig('wallExtractor', { cornerAngleThreshold: v })} />
          <SliderControl label="Min Wall Bins" value={config.wallExtractor.minWallBins} min={3} max={30} step={1} unit=""
            onChange={(v) => onUpdateConfig('wallExtractor', { minWallBins: v })} />
        </Section>

        <div style={{ borderTop: '1px solid var(--border-dim)' }} />

        {/* Calibration */}
        <div className="px-5 py-4 space-y-3">
          <div className="flex items-center justify-between">
             <span className="text-[10px] font-bold uppercase tracking-widest" style={{ color: 'var(--text-primary)' }}>Calibration</span>
             <span className="text-[9px] font-mono" style={{ color: 'var(--text-dim)' }}>GRID SEARCH</span>
          </div>
          <button 
            onClick={onRunCalibration}
            disabled={isCalibrating}
            className="w-full py-2.5 rounded-xl text-[9px] font-black uppercase tracking-widest transition-all active:scale-[0.98] flex items-center justify-center gap-2"
            style={{
              background: isCalibrating ? 'rgba(34, 211, 238, 0.1)' : 'rgba(34, 211, 238, 0.06)',
              border: '1px solid rgba(34, 211, 238, 0.15)',
              color: 'var(--accent-cyan)',
              opacity: isCalibrating ? 0.7 : 1,
            }}>
            {isCalibrating ? 'Calibrating...' : 'Run Auto Calibration'}
          </button>
          {isCalibrating && (
             <div className="w-full h-1 mt-2 rounded-full overflow-hidden" style={{ background: 'var(--bg-elevated)' }}>
               <div className="h-full transition-all duration-300" style={{ background: 'var(--accent-cyan)', width: `${calibrationProgress * 100}%` }} />
             </div>
          )}
        </div>

        <div style={{ borderTop: '1px solid var(--border-dim)' }} />

        {/* Moving Object Detection */}
        <Section title="Moving Objects" badge="ANOMALY" defaultOpen={false}>
          <ToggleControl label="Enabled" subtitle="Detect changes from profile" checked={config.movingObjectDetector.enabled}
            onChange={(v) => onUpdateConfig('movingObjectDetector', { enabled: v })} />
          <SliderControl label="Distance Threshold" value={config.movingObjectDetector.distanceThreshold} min={25} max={300} step={25} unit="mm"
            onChange={(v) => onUpdateConfig('movingObjectDetector', { distanceThreshold: v })} />
          <SliderControl label="Relative Ratio" value={config.movingObjectDetector.relativeThreshold} min={0.05} max={0.5} step={0.05} unit=""
            onChange={(v) => onUpdateConfig('movingObjectDetector', { relativeThreshold: v })} />
          <SliderControl label="Min Object Points" value={config.movingObjectDetector.minObjectPoints} min={2} max={20} step={1} unit=""
            onChange={(v) => onUpdateConfig('movingObjectDetector', { minObjectPoints: v })} />
          <ToggleControl label="Only Closer Objects" subtitle="Ignore if farther than wall" checked={config.movingObjectDetector.onlyCloser}
            onChange={(v) => onUpdateConfig('movingObjectDetector', { onlyCloser: v })} />
        </Section>

        <div style={{ borderTop: '1px solid var(--border-dim)' }} />

        {/* Visuals */}
        <Section title="Visual Layers" defaultOpen={false}>
          <ToggleControl label="Occupancy Grid" checked={config.visuals.showGrid}
            onChange={(v) => onUpdateConfig('visuals', { showGrid: v })} />
          <ToggleControl label="Point Cloud" checked={config.visuals.showPoints}
            onChange={(v) => onUpdateConfig('visuals', { showPoints: v })} />
          <ToggleControl label="Wall Segments" checked={config.visuals.showWalls}
            onChange={(v) => onUpdateConfig('visuals', { showWalls: v })} />
          <ToggleControl label="Moving Objects" checked={config.visuals.showMovingObjects}
            onChange={(v) => onUpdateConfig('visuals', { showMovingObjects: v })} />
          <ToggleControl label="Sensor Pose" checked={config.visuals.showPose}
            onChange={(v) => onUpdateConfig('visuals', { showPose: v })} />
          <ToggleControl label="Scan Beam" checked={config.visuals.scanBeam}
            onChange={(v) => onUpdateConfig('visuals', { scanBeam: v })} />

          <SliderControl label="Wall Thickness" value={config.visuals.wallThickness} min={1} max={500} step={5} unit="mm"
            color="var(--accent-amber)"
            onChange={(v) => onUpdateConfig('visuals', { wallThickness: v })} />
          <SliderControl label="Wall Opacity" value={config.visuals.wallOpacity} min={0} max={1} step={0.05} unit=""
            color="var(--accent-amber)"
            onChange={(v) => onUpdateConfig('visuals', { wallOpacity: v })} />
        </Section>
      </div>

      {/* Footer */}
      <div className="px-5 py-4 space-y-3" style={{ borderTop: '1px solid var(--border-dim)' }}>
        <button onClick={onClearMap}
          className="w-full py-2.5 rounded-xl text-[9px] font-black uppercase tracking-widest transition-all active:scale-[0.98]"
          style={{
            background: 'rgba(239, 68, 68, 0.06)',
            border: '1px solid rgba(239, 68, 68, 0.15)',
            color: 'var(--accent-red)',
          }}>
          Reset Map Memory
        </button>

        <div className="flex items-center justify-between px-1">
          <div className="flex items-center gap-2">
            <div className={`w-1.5 h-1.5 rounded-full ${status === 'connected' ? 'animate-pulse-glow' : ''}`}
              style={{ background: currentStatus.color }} />
            <span className="text-[9px] font-bold uppercase tracking-widest" style={{ color: 'var(--text-dim)' }}>Hardware</span>
          </div>
          <span className="text-[9px] font-bold uppercase tracking-wider px-2 py-0.5 rounded"
            style={{ background: currentStatus.bg, color: currentStatus.color }}>
            {currentStatus.label}
          </span>
        </div>
      </div>
    </aside>
  );
}
