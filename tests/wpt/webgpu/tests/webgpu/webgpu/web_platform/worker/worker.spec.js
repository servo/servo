/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests WebGPU is available in a dedicated worker and a shared worker.

Note: Any CTS test can be run in a worker by passing ?worker=dedicated, ?worker=shared,
?worker=service as a query parameter. The tests in this file are specifically to check
that WebGPU is available in each worker type. When run in combination with a ?worker flag,
they will test workers created from other workers (where APIs exist to do so).

TODO[2]: Figure out how to make these tests run in service workers (not actually
important unless service workers gain the ability to launch other workers).
`;import { Fixture } from '../../../common/framework/fixture.js';
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { assert } from '../../../common/util/util.js';

export const g = makeTestGroup(Fixture);

const isNode = typeof process !== 'undefined' && process?.versions?.node !== undefined;

// [1]: we load worker_launcher dynamically because ts-node support
// is using commonjs which doesn't support import.meta. Further,
// we need to put the url in a string and pass the string to import
// otherwise typescript tries to parse the file which again, fails.
// worker_launcher.js is excluded in node.tsconfig.json.

// [2]: That hack does not work in Service Workers.
const isServiceWorker = globalThis.constructor.name === 'ServiceWorkerGlobalScope';

g.test('dedicated_worker').
desc(`test WebGPU is available in dedicated workers and check for basic functionality`).
fn(async (t) => {
  t.skipIf(isNode, 'node does not support 100% compatible workers');

  t.skipIf(isServiceWorker, 'Service workers do not support this import() hack'); // [2]
  const url = './worker_launcher.js';
  const { launchDedicatedWorker } = await import(url); // [1]

  const result = await launchDedicatedWorker();
  assert(result.error === undefined, `should be no error from worker but was: ${result.error}`);
});

g.test('shared_worker').
desc(`test WebGPU is available in shared workers and check for basic functionality`).
fn(async (t) => {
  t.skipIf(isNode, 'node does not support 100% compatible workers');

  t.skipIf(isServiceWorker, 'Service workers do not support this import() hack'); // [2]
  const url = './worker_launcher.js';
  const { launchSharedWorker } = await import(url); // [1]

  const result = await launchSharedWorker();
  assert(result.error === undefined, `should be no error from worker but was: ${result.error}`);
});

g.test('service_worker').
desc(`test WebGPU is available in service workers and check for basic functionality`).
fn(async (t) => {
  t.skipIf(isNode, 'node does not support 100% compatible workers');

  t.skipIf(isServiceWorker, 'Service workers do not support this import() hack'); // [2]
  const url = './worker_launcher.js';
  const { launchServiceWorker } = await import(url); // [1]

  const result = await launchServiceWorker();
  assert(result.error === undefined, `should be no error from worker but was: ${result.error}`);
});