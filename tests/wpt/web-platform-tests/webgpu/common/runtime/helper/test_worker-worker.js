/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

import { TestLoader } from '../../framework/loader.js';
import { Logger } from '../../framework/logger.js';
import { assert } from '../../framework/util/util.js';
// should be DedicatedWorkerGlobalScope
const log = new Logger();
const loader = new TestLoader();

self.onmessage = async ev => {
  const {
    query,
    debug
  } = ev.data;
  const files = Array.from((await loader.loadTests([query])));
  assert(files.length === 1, 'worker query resulted in != 1 files');
  const f = files[0];
  const [rec] = log.record(f.id);
  assert('g' in f.spec, 'worker query resulted in README');
  const cases = Array.from(f.spec.g.iterate(rec));
  assert(cases.length === 1, 'worker query resulted in != 1 cases');
  const c = cases[0];
  const result = await c.run(debug);
  self.postMessage({
    query,
    result
  });
};
//# sourceMappingURL=test_worker-worker.js.map