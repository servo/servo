/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert } from '../util/util.js';



































































/** Test configuration options. Globally modifiable global state. */
export const globalTestConfig = {
  enableDebugLogs: false,
  maxSubcasesInFlight: 100,
  subcasesBetweenAttemptingGC: 5000,
  casesBetweenReplacingDevice: Infinity,
  testHeartbeatCallback: () => {},
  noRaceWithRejectOnTimeout: false,
  unrollConstEvalLoops: false,
  compatibility: false,
  forceFallbackAdapter: false,
  enforceDefaultLimits: false,
  blockAllFeatures: false,
  logToWebSocket: false
};

// Check if a device is a compatibility device.
// Note: The CTS generally, requires that if globalTestConfig.compatibility
// is true then the device MUST be a compatibility device since the CTS
// is trying to test that compatibility devices have the correct validation.
export function isCompatibilityDevice(device) {
  if (globalTestConfig.compatibility) {
    assert(!device.features.has('core-features-and-limits'));
  }
  return globalTestConfig.compatibility;
}