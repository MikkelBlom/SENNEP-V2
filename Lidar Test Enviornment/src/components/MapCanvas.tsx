"use client"

import React, { useEffect, useRef, useState, useCallback } from 'react';

interface MapCanvasProps {
  mapData: {
    points: { x: number; y: number; angle: number; distance: number }[];
    walls: { id: number; start: { x: number; y: number }; end: { x: number; y: number }; length: number; angle: number; confidence: number; rmsDev?: number }[];
    movingObjects?: { id: number; center: { x: number; y: number }; radius: number; avgDiff: number; points: { x: number; y: number }[]; trail: { x: number; y: number }[] }[];
    grid: { cells: { x: number; y: number; probability: number }[]; cellSize: number };
    pose: { x: number; y: number; theta: number };
  } | null;
  zoom: number;
  offset: { x: number; y: number };
  setOffset: (o: { x: number; y: number }) => void;
  setZoom: (z: number) => void;
  config: {
    showGrid: boolean;
    showPoints: boolean;
    showWalls: boolean;
    showMovingObjects: boolean;
    showPose: boolean;
    wallThickness: number;
    wallOpacity: number;
    scanBeam: boolean;
  };
  isPaused: boolean;
}

/**
 * MapCanvas: High-performance canvas renderer for the SLAM map.
 * Renders: Occupancy grid cells, RANSAC wall segments, filtered points, sensor pose.
 */
export default function MapCanvas({ mapData, zoom, offset, setOffset, setZoom, config, isPaused }: MapCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  // High-frequency data refs (avoid re-renders in the animation loop)
  const dataRef = useRef({ mapData, zoom, offset, config, isPaused });
  useEffect(() => {
    dataRef.current = { mapData, zoom, offset, config, isPaused };
  }, [mapData, zoom, offset, config, isPaused]);

  // Scan angle tracking for the sweep beam
  const scanAngleRef = useRef(0);
  useEffect(() => {
    if (mapData?.points && mapData.points.length > 0) {
      const lastPoint = mapData.points[mapData.points.length - 1];
      scanAngleRef.current = lastPoint.angle;
    }
  }, [mapData?.points]);

  // --- Interaction State ---
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });
  const [isMeasuring, setIsMeasuring] = useState(false);
  const [measureStart, setMeasureStart] = useState<{ x: number; y: number } | null>(null);
  const [measureEnd, setMeasureEnd] = useState<{ x: number; y: number } | null>(null);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    const rect = canvasRef.current?.getBoundingClientRect();
    if (!rect) return;
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    if (e.button === 1) {
      // Middle mouse: Pan
      setIsDragging(true);
      setDragStart({ x, y });
      e.preventDefault();
    } else if (e.button === 0) {
      // Left mouse: Measure
      setIsMeasuring(true);
      setMeasureStart({ x, y });
      setMeasureEnd({ x, y });
    }
  }, []);

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    const rect = canvasRef.current?.getBoundingClientRect();
    if (!rect) return;
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    if (isDragging) {
      const dx = x - dragStart.x;
      const dy = y - dragStart.y;
      setOffset({ x: offset.x + dx, y: offset.y + dy });
      setDragStart({ x, y });
    } else if (isMeasuring) {
      setMeasureEnd({ x, y });
    }
  }, [isDragging, isMeasuring, dragStart, offset, setOffset]);

  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
    setIsMeasuring(false);
    setMeasureStart(null);
    setMeasureEnd(null);
  }, []);

  const handleWheel = useCallback((e: React.WheelEvent) => {
    const scaleFactor = 1.1;
    const newZoom = e.deltaY > 0 ? zoom * scaleFactor : zoom / scaleFactor;
    setZoom(Math.round(Math.min(35000, Math.max(500, newZoom))));
  }, [zoom, setZoom]);

  // --- Render Loop ---
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d', { alpha: false });
    if (!ctx) return;

    let rafId: number;

    const render = () => {
      const { mapData: data, zoom: curZoom, offset: curOffset, config: cfg, isPaused: paused } = dataRef.current;

      const dpr = window.devicePixelRatio || 1;
      const w = canvas.clientWidth;
      const h = canvas.clientHeight;

      if (canvas.width !== w * dpr || canvas.height !== h * dpr) {
        canvas.width = w * dpr;
        canvas.height = h * dpr;
        ctx.setTransform(1, 0, 0, 1, 0, 0);
        ctx.scale(dpr, dpr);
      }

      const centerX = w / 2 + curOffset.x;
      const centerY = h / 2 + curOffset.y;
      const scale = Math.min(w, h) / curZoom;

      // World space to screen space
      const toScreen = (worldX: number, worldY: number) => ({
        x: centerX + worldX * scale,
        y: centerY + worldY * scale,
      });

      // --- Background ---
      ctx.fillStyle = '#080c16';
      ctx.fillRect(0, 0, w, h);

      // --- Grid Lines ---
      ctx.save();
      ctx.translate(centerX, centerY);

      const gridSpacingMm = 500;
      const gridSpacingPx = gridSpacingMm * scale;

      if (gridSpacingPx > 8) {
        ctx.strokeStyle = 'rgba(30, 41, 59, 0.5)';
        ctx.lineWidth = 0.5;

        const gridRange = Math.max(w, h);
        const startX = -(gridRange) - (centerX % gridSpacingPx);
        const startY = -(gridRange) - (centerY % gridSpacingPx);

        for (let x = startX; x < gridRange; x += gridSpacingPx) {
          ctx.beginPath();
          ctx.moveTo(x, -gridRange);
          ctx.lineTo(x, gridRange);
          ctx.stroke();
        }
        for (let y = startY; y < gridRange; y += gridSpacingPx) {
          ctx.beginPath();
          ctx.moveTo(-gridRange, y);
          ctx.lineTo(gridRange, y);
          ctx.stroke();
        }
      }

      // --- Range Rings ---
      const rings = [1000, 2000, 5000, 10000];
      for (const r of rings) {
        const radiusPx = r * scale;
        if (radiusPx < 10 || radiusPx > w * 3) continue;

        ctx.beginPath();
        ctx.arc(0, 0, radiusPx, 0, Math.PI * 2);
        ctx.strokeStyle = r % 5000 === 0 ? 'rgba(34, 211, 238, 0.12)' : 'rgba(34, 211, 238, 0.06)';
        ctx.lineWidth = r % 5000 === 0 ? 1 : 0.5;
        ctx.setLineDash(r % 5000 === 0 ? [] : [4, 8]);
        ctx.stroke();

        // Label
        if (r === 1000 || r % 5000 === 0) {
          ctx.fillStyle = 'rgba(34, 211, 238, 0.4)';
          ctx.font = '600 9px Inter, sans-serif';
          ctx.fillText(`${r / 1000}m`, radiusPx + 4, -4);
        }
      }
      ctx.setLineDash([]);

      // --- Dead Zone ---
      const { minRange } = dataRef.current.config as any;
      const deadZoneRadius = (minRange || 150) * scale;
      if (deadZoneRadius > 2) {
        ctx.beginPath();
        ctx.arc(0, 0, deadZoneRadius, 0, Math.PI * 2);
        ctx.fillStyle = 'rgba(239, 68, 68, 0.04)';
        ctx.fill();
        ctx.strokeStyle = 'rgba(239, 68, 68, 0.15)';
        ctx.lineWidth = 1;
        ctx.stroke();
      }

      ctx.restore();

      if (!data) {
        // No data yet — draw waiting state
        ctx.fillStyle = 'rgba(148, 163, 184, 0.3)';
        ctx.font = '600 14px Inter, sans-serif';
        ctx.textAlign = 'center';
        ctx.fillText('Waiting for LiDAR data...', w / 2, h / 2);
        rafId = requestAnimationFrame(render);
        return;
      }

      // --- Scanning Beam ---
      if (cfg.scanBeam && !paused) {
        const angleRad = (scanAngleRef.current - 90) * (Math.PI / 180);
        ctx.save();
        ctx.translate(centerX, centerY);
        ctx.beginPath();
        ctx.moveTo(0, 0);
        ctx.arc(0, 0, 12000 * scale, angleRad - 0.08, angleRad + 0.08);
        ctx.closePath();
        const gradient = ctx.createRadialGradient(0, 0, 0, 0, 0, 12000 * scale);
        gradient.addColorStop(0, 'rgba(34, 211, 238, 0.15)');
        gradient.addColorStop(1, 'rgba(34, 211, 238, 0)');
        ctx.fillStyle = gradient;
        ctx.fill();
        ctx.restore();
      }

      // --- Occupancy Grid Cells ---
      if (cfg.showGrid && data.grid?.cells) {
        const cellSizePx = (data.grid.cellSize || 50) * scale;

        for (const cell of data.grid.cells) {
          const screen = toScreen(cell.x, cell.y);

          // Color by probability: low = dim cyan, high = bright cyan
          const alpha = Math.min(0.8, Math.max(0.05, (cell.probability - 0.3) * 1.5));
          ctx.fillStyle = `rgba(34, 211, 238, ${alpha})`;
          ctx.fillRect(
            screen.x - cellSizePx / 2,
            screen.y - cellSizePx / 2,
            cellSizePx - 1,
            cellSizePx - 1
          );
        }
      }

      // --- Filtered Points ---
      if (cfg.showPoints && data.points) {
        ctx.fillStyle = 'rgba(59, 130, 246, 0.6)';
        for (const point of data.points) {
          const screen = toScreen(point.x, point.y);
          ctx.beginPath();
          ctx.arc(screen.x, screen.y, 1.5, 0, Math.PI * 2);
          ctx.fill();
        }
      }

      // --- Moving Objects ---
      if (cfg.showMovingObjects && data.movingObjects) {
        for (const obj of data.movingObjects) {
          const screenCenter = toScreen(obj.center.x, obj.center.y);
          const screenRadius = obj.radius * scale;

          // --- Trail ---
          if (obj.trail && obj.trail.length > 1) {
            const n = obj.trail.length;
            for (let i = 0; i < n; i++) {
              const age = (i + 1) / n; // 0 = oldest, 1 = newest
              const tp = toScreen(obj.trail[i].x, obj.trail[i].y);
              ctx.fillStyle = `rgba(192, 132, 252, ${age * 0.75})`; // purple-400, fades with age
              const dotR = Math.max(1.5, 4 * age);
              ctx.beginPath();
              ctx.arc(tp.x, tp.y, dotR, 0, Math.PI * 2);
              ctx.fill();
            }
          }

          // Glowing purple bounding circle
          ctx.fillStyle = 'rgba(168, 85, 247, 0.2)'; // purple-500
          ctx.strokeStyle = 'rgba(168, 85, 247, 0.8)';
          ctx.lineWidth = 1.5;

          ctx.beginPath();
          ctx.arc(screenCenter.x, screenCenter.y, Math.max(5, screenRadius), 0, Math.PI * 2);
          ctx.fill();
          ctx.stroke();

          // Object points inside
          ctx.fillStyle = 'rgba(216, 180, 254, 1)'; // purple-300
          for (const p of obj.points) {
            const sp = toScreen(p.x, p.y);
            ctx.fillRect(sp.x - 1.5, sp.y - 1.5, 3, 3);
          }

          // Label
          if (scale > 0.5) {
            ctx.fillStyle = 'rgba(168, 85, 247, 0.9)';
            ctx.font = '700 9px Inter, sans-serif';
            ctx.textAlign = 'center';
            ctx.fillText(`OBJ-${obj.id} (${Math.round(obj.avgDiff)}mm)`, screenCenter.x, screenCenter.y - screenRadius - 4);
          }
        }
      }

      // --- Wall Segments ---
      if (cfg.showWalls && data.walls) {
        for (const wall of data.walls) {
          const screenStart = toScreen(wall.start.x, wall.start.y);
          const screenEnd = toScreen(wall.end.x, wall.end.y);

          const dx = screenEnd.x - screenStart.x;
          const dy = screenEnd.y - screenStart.y;
          const lenPx = Math.sqrt(dx * dx + dy * dy);

          if (lenPx < 3) continue;

          // Wall normal (for thickness direction, pointing inward)
          let nx = -dy / lenPx;
          let ny = dx / lenPx;
          if (nx * (screenStart.x + screenEnd.x - 2 * centerX) / 2 + ny * (screenStart.y + screenEnd.y - 2 * centerY) / 2 < 0) {
            nx = -nx;
            ny = -ny;
          }

          const thickness = Math.max(2, cfg.wallThickness * scale);

          // Wall opacity scales with confidence (fades in as wall is confirmed)
          const wallConfidence = (wall as any).confidence || 1.0;
          const effectiveOpacity = cfg.wallOpacity * Math.min(1.0, wallConfidence);

          // Wall body
          ctx.beginPath();
          ctx.moveTo(screenStart.x, screenStart.y);
          ctx.lineTo(screenEnd.x, screenEnd.y);
          ctx.lineTo(screenEnd.x + nx * thickness, screenEnd.y + ny * thickness);
          ctx.lineTo(screenStart.x + nx * thickness, screenStart.y + ny * thickness);
          ctx.closePath();
          ctx.fillStyle = `rgba(245, 158, 11, ${effectiveOpacity})`;
          ctx.fill();

          // Wall inner edge (bright line)
          ctx.beginPath();
          ctx.moveTo(screenStart.x, screenStart.y);
          ctx.lineTo(screenEnd.x, screenEnd.y);
          ctx.strokeStyle = `rgba(245, 158, 11, ${Math.min(1, cfg.wallOpacity + 0.3)})`;
          ctx.lineWidth = 1.5;
          ctx.stroke();

          // Length label
          if (wall.length > 400) {
            const labelX = (screenStart.x + screenEnd.x) / 2 + nx * (thickness + 14);
            const labelY = (screenStart.y + screenEnd.y) / 2 + ny * (thickness + 14);
            ctx.fillStyle = 'rgba(245, 158, 11, 0.8)';
            ctx.font = '700 9px Inter, sans-serif';
            ctx.textAlign = 'center';
            ctx.fillText(`${Math.round(wall.length)}mm`, labelX, labelY);
          }
        }
      }

      // --- Sensor Pose Indicator ---
      if (cfg.showPose && data.pose) {
        ctx.save();
        ctx.translate(centerX, centerY);

        // Outer ring
        ctx.beginPath();
        ctx.arc(0, 0, 10, 0, Math.PI * 2);
        ctx.fillStyle = '#080c16';
        ctx.fill();
        ctx.strokeStyle = 'rgba(34, 211, 238, 0.8)';
        ctx.lineWidth = 2;
        ctx.stroke();

        // Inner dot
        ctx.beginPath();
        ctx.arc(0, 0, 4, 0, Math.PI * 2);
        ctx.fillStyle = '#22d3ee';
        ctx.fill();

        // Heading indicator
        const headingRad = data.pose.theta;
        ctx.beginPath();
        ctx.moveTo(0, 0);
        ctx.lineTo(Math.cos(headingRad) * 18, Math.sin(headingRad) * 18);
        ctx.strokeStyle = 'rgba(34, 211, 238, 0.5)';
        ctx.lineWidth = 1.5;
        ctx.stroke();

        ctx.restore();
      }

      // --- Measurement Tool (screen-space, after all transforms) ---
      if (isMeasuring && measureStart && measureEnd) {
        const mdx = measureEnd.x - measureStart.x;
        const mdy = measureEnd.y - measureStart.y;
        const distPx = Math.sqrt(mdx * mdx + mdy * mdy);
        const distMm = distPx / scale;

        ctx.beginPath();
        ctx.setLineDash([5, 5]);
        ctx.strokeStyle = '#ef4444';
        ctx.lineWidth = 1.5;
        ctx.moveTo(measureStart.x, measureStart.y);
        ctx.lineTo(measureEnd.x, measureEnd.y);
        ctx.stroke();
        ctx.setLineDash([]);

        // Label
        const mx = (measureStart.x + measureEnd.x) / 2;
        const my = (measureStart.y + measureEnd.y) / 2;
        const text = `${Math.round(distMm)}mm`;
        ctx.font = '700 11px Inter, sans-serif';
        const metrics = ctx.measureText(text);
        ctx.fillStyle = 'rgba(239, 68, 68, 0.9)';
        ctx.fillRect(mx - metrics.width / 2 - 6, my - 18, metrics.width + 12, 20);
        ctx.fillStyle = '#ffffff';
        ctx.textAlign = 'center';
        ctx.fillText(text, mx, my - 4);
      }

      rafId = requestAnimationFrame(render);
    };

    rafId = requestAnimationFrame(render);
    return () => cancelAnimationFrame(rafId);
  }, [isMeasuring, measureStart, measureEnd]);

  return (
    <canvas
      ref={canvasRef}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onMouseLeave={handleMouseUp}
      onWheel={handleWheel}
      onContextMenu={(e) => e.preventDefault()}
      className="w-full h-full block cursor-crosshair"
      style={{ background: 'var(--bg-deep)' }}
    />
  );
}
