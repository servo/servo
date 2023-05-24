/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Tests WebGPU is available in a worker.

Note: The CTS test can be run in a worker by passing in worker=1 as
a query parameter. This test is specifically to check that WebGPU
is available in a worker.
`;
import { Fixture } from '../../../common/framework/fixture.js';
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { assert } from '../../../common/util/util.js';

export const g = makeTestGroup(Fixture);

function isNode() {
  return typeof process !== 'undefined' && process?.versions?.node !== undefined;
}

g.test('worker')
  .desc(`test WebGPU is available in DedicatedWorkers and check for basic functionality`)
  .fn(async t => {
    if (isNode()) {
      t.skip('node does not support 100% compatible workers');
      return;
    }
    // Note: we load worker_launcher dynamically because ts-node support
    // is using commonjs which doesn't support import.meta. Further,
    // we need to put the url in a string add pass the string to import
    // otherwise typescript tries to parse the file which again, fails.
    // worker_launcher.js is excluded in node.tsconfig.json.
    const url = './worker_launcher.js';
    const { launchWorker } = await import(url);
    const result = await launchWorker();
    assert(result.error === undefined, `should be no error from worker but was: ${result.error}`);
  });
