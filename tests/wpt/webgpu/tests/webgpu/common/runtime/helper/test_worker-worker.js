/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { setBaseResourcePath } from '../../framework/resources.js';
import { DefaultTestFileLoader } from '../../internal/file_loader.js';
import { Logger } from '../../internal/logging/logger.js';
import { parseQuery } from '../../internal/query/parseQuery.js';

import { setDefaultRequestAdapterOptions } from '../../util/navigator_gpu.js';
import { assert } from '../../util/util.js';

// Should be DedicatedWorkerGlobalScope, but importing lib "webworker" conflicts with lib "dom".

const loader = new DefaultTestFileLoader();

setBaseResourcePath('../../../resources');

self.onmessage = async ev => {
  const query = ev.data.query;
  const expectations = ev.data.expectations;
  const defaultRequestAdapterOptions = ev.data.defaultRequestAdapterOptions;
  const debug = ev.data.debug;

  setDefaultRequestAdapterOptions(defaultRequestAdapterOptions);

  Logger.globalDebugMode = debug;
  const log = new Logger();

  const testcases = Array.from(await loader.loadCases(parseQuery(query)));
  assert(testcases.length === 1, 'worker query resulted in != 1 cases');

  const testcase = testcases[0];
  const [rec, result] = log.record(testcase.query.toString());
  await testcase.run(rec, expectations);

  self.postMessage({ query, result });
};
