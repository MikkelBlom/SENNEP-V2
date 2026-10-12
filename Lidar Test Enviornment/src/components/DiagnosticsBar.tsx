"use client"

import React from 'react';

interface DiagnosticsBarProps {
  status: string;
  diagnostics: {
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
    profileSweeps?: number;
  } | null;
  pose: { x: number; y: number; theta: number } | null;
}

function Metric({ label, value, unit, color }: { label: string; value: string | number; unit?: string; color?: string }) {
  return (
    <div className="flex flex-col items-center gap-0.5 px-3">
      <span className="text-[8px] font-bold uppercase tracking-widest" style={{ color: 'var(--text-dim)' }}>{label}</span>
      <div className="flex items-baseline gap-0.5">
        <span className="text-xs font-black tabular-nums" style={{ color: color || 'var(--text-secondary)' }}>{value}</span>
        {unit && <span className="text-[8px] font-bold" style={{ color: 'var(--text-dim)' }}>{unit}</span>}
      </div>
    </div>
  );
}

function Divider() {
  return <div className="w-px h-6 mx-1" style={{ background: 'var(--border-dim)' }} />;
}

export default function DiagnosticsBar({ status, diagnostics, pose }: DiagnosticsBarProps) {
  const diag = diagnostics;

  // Pipeline stages visualization
  const pipelineStages = diag ? [
    { label: 'Raw', value: diag.rawCount },
    { label: 'Filtered', value: diag.filteredCount },
    { label: 'Downsampled', value: diag.downsampledCount },
    { label: 'Cleaned', value: diag.cleanedCount },
  ] : null;

  return (
    <div
      className="flex items-center justify-between px-6 py-2.5 flex-shrink-0"
      style={{ background: 'var(--bg-surface)', borderTop: '1px solid var(--border-dim)' }}
    >
      {/* Left: Connection + Pipeline flow */}
      <div className="flex items-center gap-1">
        {/* Connection indicator */}
        <div className="flex items-center gap-1.5 mr-3">
          <div
            className={`w-1.5 h-1.5 rounded-full ${status === 'connected' ? 'animate-pulse-glow' : ''}`}
            style={{ background: status === 'connected' ? 'var(--accent-green)' : 'var(--text-dim)' }}
          />
          <span className="text-[9px] font-bold uppercase tracking-wider" style={{ color: status === 'connected' ? 'var(--accent-green)' : 'var(--text-dim)' }}>
            {status}
          </span>
        </div>

        <Divider />

        {/* Pipeline flow */}
        {pipelineStages && (
          <div className="flex items-center gap-0.5">
            {pipelineStages.map((stage, i) => (
              <React.Fragment key={stage.label}>
                <div className="flex items-center gap-1 px-2">
                  <span className="text-[8px] font-bold uppercase tracking-wider" style={{ color: 'var(--text-dim)' }}>{stage.label}</span>
                  <span className="text-[10px] font-black tabular-nums" style={{ color: 'var(--text-secondary)' }}>{stage.value}</span>
                </div>
                {i < pipelineStages.length - 1 && (
                  <svg className="w-2.5 h-2.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" style={{ color: 'var(--text-dim)' }}>
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M9 5l7 7-7 7" />
                  </svg>
                )}
              </React.Fragment>
            ))}
          </div>
        )}
      </div>

      {/* Center: Processing stats */}
      <div className="flex items-center">
        {diag && (
          <>
            <Metric label="Walls" value={diag.wallCount} color="var(--accent-amber)" />
            <Metric label="Grid" value={diag.gridCellCount} color="var(--accent-cyan)" />
            <Divider />
            <Metric label="Latency" value={diag.processTimeMs} unit="ms"
              color={diag.processTimeMs > 50 ? 'var(--accent-red)' : 'var(--accent-green)'} />
            <Metric label="ICP" value={diag.icpConverged ? '✓' : `${diag.icpIterations}it`}
              color={diag.icpConverged ? 'var(--accent-green)' : 'var(--accent-amber)'} />
            {diag.profileSweeps !== undefined && (
              <>
                <Divider />
                <Metric label="Profile" value={`${diag.profileSweeps} sweeps`}
                  color="var(--accent-purple)" />
              </>
            )}
          </>
        )}
      </div>

      {/* Right: Pose */}
      <div className="flex items-center">
        {pose && (
          <>
            <Metric label="X" value={Math.round(pose.x)} unit="mm" />
            <Metric label="Y" value={Math.round(pose.y)} unit="mm" />
            <Metric label="θ" value={(pose.theta * 180 / Math.PI).toFixed(1)} unit="°" />
          </>
        )}
        {diag && (
          <>
            <Divider />
            <Metric label="Sweep" value={`#${diag.sweepNumber}`} />
          </>
        )}
      </div>
    </div>
  );
}
