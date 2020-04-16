/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

import { TestLoader } from '../framework/loader.js';
import { Logger } from '../framework/logger.js';
import { makeQueryString } from '../framework/url_query.js';
import { AsyncMutex } from '../framework/util/async_mutex.js';
import { optionEnabled } from './helper/options.js';
import { TestWorker } from './helper/test_worker.js';

(async () => {
  const loader = new TestLoader();
  const files = await loader.loadTestsFromQuery(window.location.search);
  const worker = optionEnabled('worker') ? new TestWorker() : undefined;
  const log = new Logger();
  const mutex = new AsyncMutex();
  const running = [];

  for (const f of files) {
    if (!('g' in f.spec)) {
      continue;
    }

    const [rec] = log.record(f.id);

    for (const t of f.spec.g.iterate(rec)) {
      const name = makeQueryString(f.id, t.id); // Note: apparently, async_tests must ALL be added within the same task.

      async_test(function () {
        const p = mutex.with(async () => {
          let r;

          if (worker) {
            r = await worker.run(name);
            t.injectResult(r);
          } else {
            r = await t.run();
          }

          this.step(() => {
            // Unfortunately, it seems not possible to surface any logs for warn/skip.
            if (r.status === 'fail') {
              throw (r.logs || []).map(s => s.toJSON()).join('\n\n');
            }
          });
          this.done();
        });
        running.push(p);
        return p;
      }, name);
    }
  }

  await Promise.all(running);
  const resultsElem = document.getElementById('results');
  resultsElem.textContent = log.asJSON(2);
})();
//# sourceMappingURL=wpt.js.map