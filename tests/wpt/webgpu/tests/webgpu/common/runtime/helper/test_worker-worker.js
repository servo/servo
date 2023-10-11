/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { setBaseResourcePath } from '../../framework/resources.js';
import { globalTestConfig } from '../../framework/test_config.js';
import { DefaultTestFileLoader } from '../../internal/file_loader.js';
import { Logger } from '../../internal/logging/logger.js';
import { parseQuery } from '../../internal/query/parseQuery.js';

import { setDefaultRequestAdapterOptions } from '../../util/navigator_gpu.js';
import { assert } from '../../util/util.js';

const loader = new DefaultTestFileLoader();

setBaseResourcePath('../../../resources');

self.onmessage = async ev => {
  const query = ev.data.query;
  const expectations = ev.data.expectations;
  const ctsOptions = ev.data.ctsOptions;

  const { debug, unrollConstEvalLoops, powerPreference, compatibility } = ctsOptions;
  globalTestConfig.unrollConstEvalLoops = unrollConstEvalLoops;
  globalTestConfig.compatibility = compatibility;

  Logger.globalDebugMode = debug;
  const log = new Logger();

  if (powerPreference || compatibility) {
    setDefaultRequestAdapterOptions({
      ...(powerPreference && { powerPreference }),
      // MAINTENANCE_TODO: Change this to whatever the option ends up being
      ...(compatibility && { compatibilityMode: true }),
    });
  }

  const testcases = Array.from(await loader.loadCases(parseQuery(query)));
  assert(testcases.length === 1, 'worker query resulted in != 1 cases');

  const testcase = testcases[0];
  const [rec, result] = log.record(testcase.query.toString());
  await testcase.run(rec, expectations);

  self.postMessage({ query, result });
};
