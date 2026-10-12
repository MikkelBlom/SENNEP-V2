/**
 * SensorFusion: Extended Kalman Filter (EKF) stub.
 *
 * State vector: [x, y, theta, vx, vy, omega]
 *
 * Currently acts as a passthrough for the ICP pose. When IMU and wheel
 * encoders are added, this module will fuse their measurements with
 * the LiDAR-based pose estimate to produce a more robust position.
 *
 * The interface is fully defined — only the prediction/update matrices
 * need to be filled when hardware arrives.
 */
class SensorFusion {
  constructor(config = {}) {
    this.config = {
      enabled: false,  // Passthrough mode until IMU/wheels are connected
      ...config,
    };

    // EKF state: [x, y, theta, vx, vy, omega]
    this.state = new Float64Array(6);

    // Covariance matrix (6x6, flattened)
    this.covariance = new Float64Array(36);
    this.initCovariance();

    this.lastTimestamp = null;
  }

  updateConfig(newConfig) {
    Object.assign(this.config, newConfig);
  }

  initCovariance() {
    // Diagonal initialization with high uncertainty
    for (let i = 0; i < 6; i++) {
      this.covariance[i * 6 + i] = 1000.0;
    }
  }

  /**
   * Main update: Takes ICP pose and returns fused estimate.
   * @param {{x: number, y: number, theta: number}} icpPose
   * @param {number} timestamp
   * @returns {{x: number, y: number, theta: number}}
   */
  process(icpPose, timestamp = Date.now()) {
    if (!this.config.enabled) {
      // Passthrough: just return ICP pose directly
      return { ...icpPose };
    }

    const dt = this.lastTimestamp ? (timestamp - this.lastTimestamp) / 1000 : 0.1;
    this.lastTimestamp = timestamp;

    // Prediction step (dead reckoning from velocity estimates)
    this.predict(dt);

    // Update step (correct using ICP measurement)
    this.updateFromLidar(icpPose);

    return {
      x: this.state[0],
      y: this.state[1],
      theta: this.state[2],
    };
  }

  /**
   * Prediction: Propagate state forward using velocity model.
   * With no IMU, velocities decay toward zero.
   */
  predict(dt) {
    // Simple constant-velocity model
    this.state[0] += this.state[3] * dt; // x += vx * dt
    this.state[1] += this.state[4] * dt; // y += vy * dt
    this.state[2] += this.state[5] * dt; // theta += omega * dt

    // Velocity decay (no IMU = assume decelerating)
    this.state[3] *= 0.95;
    this.state[4] *= 0.95;
    this.state[5] *= 0.95;

    // Increase uncertainty (process noise)
    const processNoise = [10 * dt, 10 * dt, 0.01 * dt, 50 * dt, 50 * dt, 0.1 * dt];
    for (let i = 0; i < 6; i++) {
      this.covariance[i * 6 + i] += processNoise[i];
    }
  }

  /**
   * Update from LiDAR ICP pose measurement.
   * Simplified Kalman gain calculation for position-only measurement.
   */
  updateFromLidar(icpPose) {
    // Measurement noise (how much we trust ICP)
    const measurementNoise = [5.0, 5.0, 0.005]; // x, y, theta

    // Innovation (difference between measurement and prediction)
    const innovation = [
      icpPose.x - this.state[0],
      icpPose.y - this.state[1],
      this.normalizeAngle(icpPose.theta - this.state[2]),
    ];

    // Simplified Kalman gain for diagonal measurement model
    const measIndices = [0, 1, 2]; // State indices that are directly observed
    for (let mi = 0; mi < 3; mi++) {
      const si = measIndices[mi];
      const s = this.covariance[si * 6 + si] + measurementNoise[mi];
      const k = this.covariance[si * 6 + si] / s;

      this.state[si] += k * innovation[mi];
      this.covariance[si * 6 + si] *= (1 - k);
    }
  }

  /**
   * IMU measurement input — no-op until hardware is connected.
   * @param {{ax: number, ay: number, gz: number}} imuData - Accel (m/s²) + Gyro (rad/s)
   */
  addImuMeasurement(imuData) {
    if (!this.config.enabled) return;
    // Future: Update velocity estimates from accelerometer
    // Future: Update angular velocity from gyroscope
    // this.state[5] = blend(this.state[5], imuData.gz, gain);
  }

  /**
   * Wheel encoder input — no-op until hardware is connected.
   * @param {{leftSpeed: number, rightSpeed: number}} wheelData - Wheel speeds (mm/s)
   */
  addWheelMeasurement(wheelData) {
    if (!this.config.enabled) return;
    // Future: Differential drive model → vx, omega
    // const v = (wheelData.leftSpeed + wheelData.rightSpeed) / 2;
    // const omega = (wheelData.rightSpeed - wheelData.leftSpeed) / wheelBase;
  }

  normalizeAngle(angle) {
    while (angle > Math.PI) angle -= 2 * Math.PI;
    while (angle < -Math.PI) angle += 2 * Math.PI;
    return angle;
  }

  reset() {
    this.state.fill(0);
    this.initCovariance();
    this.lastTimestamp = null;
  }
}

module.exports = SensorFusion;
