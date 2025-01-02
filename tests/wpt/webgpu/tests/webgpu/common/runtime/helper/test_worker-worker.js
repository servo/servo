/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { setBaseResourcePath } from '../../framework/resources.js';import { DefaultTestFileLoader } from '../../internal/file_loader.js';import { parseQuery } from '../../internal/query/parseQuery.js';
import { assert } from '../../util/util.js';

import { setupWorkerEnvironment } from './utils_worker.js';

// Should be WorkerGlobalScope, but importing lib "webworker" conflicts with lib "dom".



const loader = new DefaultTestFileLoader();

setBaseResourcePath('../../../resources');

async function reportTestResults(ev) {
  const { query, expectations, ctsOptions } = ev.data;

  const log = setupWorkerEnvironment(ctsOptions);

  const testcases = Array.from(await loader.loadCases(parseQuery(query)));
  assert(testcases.length === 1, 'worker query resulted in != 1 cases');

  const testcase = testcases[0];
  const [rec, result] = log.record(testcase.query.toString());
  await testcase.run(rec, expectations);

  this.postMessage({
    query,
    result: {
      ...result,
      logs: result.logs?.map((l) => l.toRawData())
    }
  });
}

self.onmessage = (ev) => {
  void reportTestResults.call(ev.source || self, ev);
};

self.onconnect = (event) => {
  const port = event.ports[0];

  port.onmessage = (ev) => {
    void reportTestResults.call(port, ev);
  };
};