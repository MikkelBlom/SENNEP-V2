const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const { SerialPort } = require('serialport');
const LidarParser = require('./lidar-parser');
const Pipeline = require('./pipeline/Pipeline');

let mainWindow;
let lidarPort;
let isConnecting = false;

const pipeline = new Pipeline({
  preFilter: { minRange: 150, maxRange: 12000, minConfidence: 100 },
  downsampler: { cellSize: 30, enabled: true },
  outlierRemoval: { enabled: true, neighborCount: 8, stddevMultiplier: 1.5 },
  occupancyGrid: { cellSize: 50, hitIncrement: 0.9, missDecrement: 0.4, raycastEnabled: true },
  scanMatcher: { enabled: false, maxIterations: 30 },
  sensorFusion: { enabled: false },
  wallExtractor: {
    binResolution: 0.5,
    emaAlpha: 0.15,
    minWallBins: 10,
    maxGapBins: 4,
    lineDeviationThreshold: 40,
    minWallLength: 200,
    cornerAngleThreshold: 25,
  },
  wallTracker: {
    smoothing: 0.10,               // EMA alpha on endpoints — low = very stable
    persistence: 45,               // Frames before an unseen wall expires
    matchDistanceThreshold: 400,   // mm — centroid distance to match across frames
    matchAngleThreshold: 20,       // degrees — angle tolerance for matching
    mergeDistanceThreshold: 200,   // mm — merge nearby parallel fragments
    mergeAngleThreshold: 12,       // degrees — angle tolerance for merging
    minLifetimeToShow: 3,          // Frames confirmed before a wall renders
  },
  updateRate: 2,
});

// --- Window ---

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1400,
    height: 900,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
    },
    title: 'Sennep SLAM Suite',
    autoHideMenuBar: true,
    backgroundColor: '#0a0e1a',
  });

  mainWindow.loadURL('http://localhost:3000');

  // Auto-detect and connect after window loads
  mainWindow.webContents.on('did-finish-load', () => {
    detectAndConnect();
  });
}

// --- IPC Handlers ---

// Port listing for the UI dropdown
ipcMain.handle('list-serial-ports', async () => {
  try {
    const ports = await SerialPort.list();
    return ports.map(p => ({
      path: p.path,
      manufacturer: p.manufacturer || 'Unknown',
      vendorId: p.vendorId,
      productId: p.productId,
    }));
  } catch (err) {
    console.error('[Main] Port list error:', err.message);
    return [];
  }
});

// Manual port selection from UI
ipcMain.handle('connect-port', async (event, portPath) => {
  if (lidarPort && lidarPort.isOpen) {
    try { lidarPort.close(); } catch (e) { /* ignore */ }
  }
  return connectToLidar(portPath);
});

// Disconnect
ipcMain.handle('disconnect-port', async () => {
  if (lidarPort && lidarPort.isOpen) {
    try { lidarPort.close(); } catch (e) { /* ignore */ }
    lidarPort = null;
  }
  if (mainWindow) mainWindow.webContents.send('lidar-status', 'disconnected');
  return true;
});

// Pipeline config updates (per-module)
ipcMain.on('update-pipeline-config', (event, { moduleId, config }) => {
  pipeline.updateConfig(moduleId, config);
});

// Lidar status
ipcMain.handle('get-lidar-status', () => {
  if (!lidarPort) return 'disconnected';
  return lidarPort.isOpen ? 'connected' : 'searching';
});

// Clear map
ipcMain.on('clear-map', () => {
  pipeline.clearMap();
});

// Recording
ipcMain.handle('start-recording', async () => {
  const recordDir = path.join(__dirname, 'recordings');
  if (!require('fs').existsSync(recordDir)) {
    require('fs').mkdirSync(recordDir, { recursive: true });
  }
  pipeline.startRecording(recordDir);
  return true;
});

ipcMain.handle('stop-recording', async () => {
  return pipeline.stopRecording();
});

// Calibration: grid-search parameter optimization using recorded data
ipcMain.handle('run-calibration', async () => {
  const CalibrationRunner = require('./pipeline/CalibrationRunner');
  const recordDir = path.join(__dirname, 'recordings');
  const fsSync = require('fs');

  // Find most recent recording
  if (!fsSync.existsSync(recordDir)) {
    return { error: 'No recordings directory found. Record some data first.' };
  }

  const files = fsSync.readdirSync(recordDir)
    .filter(f => f.endsWith('.jsonl'))
    .map(f => ({ name: f, path: path.join(recordDir, f), mtime: fsSync.statSync(path.join(recordDir, f)).mtime }))
    .sort((a, b) => b.mtime - a.mtime);

  if (files.length === 0) {
    return { error: 'No recording files found. Record some data first.' };
  }

  const runner = new CalibrationRunner();

  try {
    const result = await runner.run(
      files[0].path,
      pipeline.preFilter.config,
      (progress) => {
        if (mainWindow) {
          mainWindow.webContents.send('calibration-progress', progress);
        }
      }
    );

    // Auto-apply the best config
    pipeline.updateConfig('wallExtractor', result.bestConfig);
    if (mainWindow) {
      mainWindow.webContents.send('calibration-complete', result);
    }

    return result;
  } catch (err) {
    console.error('[Calibration] Error:', err.message);
    return { error: err.message };
  }
});

// --- LiDAR Connection ---

/**
 * Auto-detect: Scan for LDROBOT-style USB serial devices and connect
 * to the first one found. Falls back to COM4 if no auto-detect match.
 */
async function detectAndConnect() {
  if (isConnecting || (lidarPort && lidarPort.isOpen)) return;

  if (mainWindow) mainWindow.webContents.send('lidar-status', 'searching');

  try {
    const ports = await SerialPort.list();
    // Look for LDROBOT identifiers (common VID:PID patterns)
    const ldrobotPort = ports.find(p =>
      (p.manufacturer && p.manufacturer.toLowerCase().includes('silicon')) ||
      (p.vendorId && p.vendorId.toLowerCase() === '10c4') ||
      (p.manufacturer && p.manufacturer.toLowerCase().includes('cp210'))
    );

    const targetPath = ldrobotPort ? ldrobotPort.path : null;

    if (targetPath) {
      console.log(`[Main] Auto-detected LiDAR on ${targetPath}`);
      await connectToLidar(targetPath);
    } else {
      console.log('[Main] No LiDAR auto-detected. Waiting for manual selection.');
      if (mainWindow) {
        mainWindow.webContents.send('lidar-status', 'disconnected');
        mainWindow.webContents.send('available-ports', ports.map(p => ({
          path: p.path,
          manufacturer: p.manufacturer || 'Unknown',
        })));
      }
    }
  } catch (err) {
    console.error('[Main] Port scan error:', err.message);
    if (mainWindow) mainWindow.webContents.send('lidar-status', 'error');
  }
}

async function connectToLidar(portPath) {
  if (isConnecting) return false;
  isConnecting = true;

  if (mainWindow) mainWindow.webContents.send('lidar-status', 'connecting');

  return new Promise((resolve) => {
    try {
      lidarPort = new SerialPort({
        path: portPath,
        baudRate: 230400,
        autoOpen: false,
      });

      lidarPort.open((err) => {
        isConnecting = false;

        if (err) {
          console.error(`[Main] Failed to open ${portPath}:`, err.message);
          if (mainWindow) mainWindow.webContents.send('lidar-status', 'error');
          resolve(false);
          return;
        }

        console.log(`[Main] Connected to LiDAR on ${portPath}`);
        if (mainWindow) {
          mainWindow.webContents.send('lidar-status', 'connected');
          mainWindow.webContents.send('connected-port', portPath);
        }

        const parser = new LidarParser();
        lidarPort.pipe(parser);

        // Atomic 360° sweep buffering
        let sweepBuffer = [];
        let lastAngle = 0;

        parser.on('data', (rawPoints) => {
          if (rawPoints.length === 0) return;

          const firstAngle = rawPoints[0].angle;

          // Detect angle wrap (new 360° sweep)
          if (firstAngle < lastAngle - 50) {
            if (sweepBuffer.length > 0) {
              const processed = pipeline.process(sweepBuffer);
              if (processed && mainWindow && !mainWindow.isDestroyed()) {
                mainWindow.webContents.send('map-update', processed);
              }
            }
            sweepBuffer = [];
          }

          sweepBuffer.push(...rawPoints);
          lastAngle = firstAngle;
        });

        lidarPort.on('close', () => {
          console.log('[Main] LiDAR port closed');
          if (mainWindow && !mainWindow.isDestroyed()) {
            mainWindow.webContents.send('lidar-status', 'disconnected');
          }
          lidarPort = null;
        });

        lidarPort.on('error', (err) => {
          console.error('[Main] Serial error:', err.message);
          if (mainWindow && !mainWindow.isDestroyed()) {
            mainWindow.webContents.send('lidar-status', 'error');
          }
        });

        resolve(true);
      });
    } catch (err) {
      isConnecting = false;
      console.error('[Main] Connection error:', err.message);
      if (mainWindow) mainWindow.webContents.send('lidar-status', 'error');
      resolve(false);
    }
  });
}

// --- Lifecycle ---

app.whenReady().then(createWindow);

app.on('window-all-closed', () => {
  if (lidarPort && lidarPort.isOpen) {
    try { lidarPort.close(); } catch (e) { /* ignore */ }
  }
  if (process.platform !== 'darwin') app.quit();
});
