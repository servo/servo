/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { globalTestConfig } from '../../framework/test_config.js';import { Logger } from '../../internal/logging/logger.js';
import { setDefaultRequestAdapterOptions } from '../../util/navigator_gpu.js';









/**
 * Set config environment for workers with ctsOptions and return a Logger.
 */
export function setupWorkerEnvironment(ctsOptions) {
  const { powerPreference, compatibility } = ctsOptions;
  globalTestConfig.enableDebugLogs = ctsOptions.debug;
  globalTestConfig.unrollConstEvalLoops = ctsOptions.unrollConstEvalLoops;
  globalTestConfig.compatibility = compatibility;
  globalTestConfig.enforceDefaultLimits = ctsOptions.enforceDefaultLimits;
  globalTestConfig.blockAllFeatures = ctsOptions.blockAllFeatures;
  if (ctsOptions.subcasesBetweenAttemptingGC) {
    globalTestConfig.subcasesBetweenAttemptingGC = Number(ctsOptions.subcasesBetweenAttemptingGC);
  }
  if (ctsOptions.casesBetweenReplacingDevice) {
    globalTestConfig.casesBetweenReplacingDevice = Number(ctsOptions.casesBetweenReplacingDevice);
  }
  globalTestConfig.logToWebSocket = ctsOptions.logToWebSocket;

  const log = new Logger();

  if (powerPreference || compatibility) {
    setDefaultRequestAdapterOptions({
      ...(powerPreference && { powerPreference }),
      ...(compatibility && { featureLevel: 'compatibility' })
    });
  }

  return log;
}