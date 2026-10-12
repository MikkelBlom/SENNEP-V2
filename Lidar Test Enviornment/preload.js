const { contextBridge, ipcRenderer } = require('electron');

/**
 * Secure IPC bridge for the SLAM Suite renderer process.
 */
contextBridge.exposeInMainWorld('api', {
  // --- LiDAR Data ---
  onMapUpdate: (callback) => {
    const listener = (event, data) => callback(data);
    ipcRenderer.on('map-update', listener);
    return () => ipcRenderer.removeListener('map-update', listener);
  },

  // --- Connection Status ---
  onLidarStatus: (callback) => {
    const listener = (event, status) => callback(status);
    ipcRenderer.on('lidar-status', listener);
    return () => ipcRenderer.removeListener('lidar-status', listener);
  },

  onConnectedPort: (callback) => {
    const listener = (event, port) => callback(port);
    ipcRenderer.on('connected-port', listener);
    return () => ipcRenderer.removeListener('connected-port', listener);
  },

  onAvailablePorts: (callback) => {
    const listener = (event, ports) => callback(ports);
    ipcRenderer.on('available-ports', listener);
    return () => ipcRenderer.removeListener('available-ports', listener);
  },

  // --- Commands ---
  getStatus: () => ipcRenderer.invoke('get-lidar-status'),
  listPorts: () => ipcRenderer.invoke('list-serial-ports'),
  connectPort: (portPath) => ipcRenderer.invoke('connect-port', portPath),
  disconnectPort: () => ipcRenderer.invoke('disconnect-port'),

  // --- Pipeline Config ---
  updatePipelineConfig: (moduleId, config) => {
    ipcRenderer.send('update-pipeline-config', { moduleId, config });
  },

  // --- Map Actions ---
  clearMap: () => ipcRenderer.send('clear-map'),

  // --- Recording ---
  startRecording: () => ipcRenderer.invoke('start-recording'),
  stopRecording: () => ipcRenderer.invoke('stop-recording'),

  // --- Calibration ---
  runCalibration: () => ipcRenderer.invoke('run-calibration'),
  onCalibrationProgress: (callback) => {
    const listener = (event, data) => callback(data);
    ipcRenderer.on('calibration-progress', listener);
    return () => ipcRenderer.removeListener('calibration-progress', listener);
  },
  onCalibrationComplete: (callback) => {
    const listener = (event, data) => callback(data);
    ipcRenderer.on('calibration-complete', listener);
    return () => ipcRenderer.removeListener('calibration-complete', listener);
  },
});
