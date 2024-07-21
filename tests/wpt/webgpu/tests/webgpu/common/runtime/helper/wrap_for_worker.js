/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { LogMessageWithStack } from '../../internal/logging/log_message.js';import { comparePaths, comparePublicParamsPaths, Ordering } from '../../internal/query/compare.js';
import { parseQuery } from '../../internal/query/parseQuery.js';
import { TestQuerySingleCase } from '../../internal/query/query.js';

import { assert } from '../../util/util.js';

import { setupWorkerEnvironment } from './utils_worker.js';

/**
 * Sets up the currently running Web Worker to wrap the TestGroup object `g`.
 * `g` is the `g` exported from a `.spec.ts` file: a TestGroupBuilder<F> interface,
 * which underneath is actually a TestGroup<F> object.
 *
 * This is used in the generated `.as_worker.js` files that are generated to use as service workers.
 */
export function wrapTestGroupForWorker(g) {
  self.onmessage = async (ev) => {
    const { query, expectations, ctsOptions } = ev.data;
    try {
      const log = setupWorkerEnvironment(ctsOptions);

      const testQuery = parseQuery(query);
      assert(testQuery instanceof TestQuerySingleCase);
      let testcase = null;
      for (const t of g.iterate()) {
        if (comparePaths(t.testPath, testQuery.testPathParts) !== Ordering.Equal) {
          continue;
        }
        for (const c of t.iterate(testQuery.params)) {
          if (comparePublicParamsPaths(c.id.params, testQuery.params) === Ordering.Equal) {
            testcase = c;
          }
        }
      }
      assert(!!testcase, 'testcase not found');
      const [rec, result] = log.record(query);
      await testcase.run(rec, testQuery, expectations);

      ev.source?.postMessage({ query, result });
    } catch (thrown) {
      const ex = thrown instanceof Error ? thrown : new Error(`${thrown}`);
      ev.source?.postMessage({
        query,
        result: {
          status: 'fail',
          timems: 0,
          logs: [LogMessageWithStack.wrapError('INTERNAL', ex)]
        }
      });
    }
  };
}