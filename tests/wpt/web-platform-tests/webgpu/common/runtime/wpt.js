/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

import { DefaultTestFileLoader } from '../framework/file_loader.js';
import { Logger } from '../framework/logging/logger.js';
import { AsyncMutex } from '../framework/util/async_mutex.js';
import { assert } from '../framework/util/util.js';
import { optionEnabled } from './helper/options.js';
import { TestWorker } from './helper/test_worker.js';

(async () => {
  const loader = new DefaultTestFileLoader();
  const qs = new URLSearchParams(window.location.search).getAll('q');
  assert(qs.length === 1, 'currently, there must be exactly one ?q=');
  const testcases = await loader.loadTests(qs[0]);
  await addWPTTests(testcases);
})(); // Note: async_tests must ALL be added within the same task. This function *must not* be async.


function addWPTTests(testcases) {
  const worker = optionEnabled('worker') ? new TestWorker(false) : undefined;
  const log = new Logger(false);
  const mutex = new AsyncMutex();
  const running = [];

  for (const testcase of testcases) {
    const name = testcase.query.toString();

    const wpt_fn = function () {
      const p = mutex.with(async () => {
        const [rec, res] = log.record(name);

        if (worker) {
          await worker.run(rec, name);
        } else {
          await testcase.run(rec);
        }

        this.step(() => {
          // Unfortunately, it seems not possible to surface any logs for warn/skip.
          if (res.status === 'fail') {
            throw (res.logs || []).map(s => s.toJSON()).join('\n\n');
          }
        });
        this.done();
      });
      running.push(p);
      return p;
    };

    async_test(wpt_fn, name);
  }

  return Promise.all(running).then(() => log);
}
//# sourceMappingURL=wpt.js.map