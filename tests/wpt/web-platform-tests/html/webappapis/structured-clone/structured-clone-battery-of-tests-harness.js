/**
 * Runs a collection of tests that determine if an API implements structured clone
 * correctly.
 *
 * The `runner` parameter has the following properties:
 * - `setup()`: An optional function run once before testing starts
 * - `teardown()`: An option function run once after all tests are done
 * - `preTest()`: An optional, async function run before a test
 * - `postTest()`: An optional, async function run after a test is done
 * - `structuredClone(obj, transferList)`: Required function that somehow
 *                                         structurally clones an object.
 * - `noTransferTests`: When true, disables tests with transferables
 */

function runStructuredCloneBatteryOfTests(runner) {
  const defaultRunner = {
    setup() {},
    preTest() {},
    postTest() {},
    teardown() {}
  };
  runner = Object.assign({}, defaultRunner, runner);

  let setupPromise = runner.setup();
  const allTests = structuredCloneBatteryOfTests.map(test => {

    return new Promise(resolve => {
      promise_test(async _ => {
        test = await test;
        await setupPromise;
        await runner.preTest(test);
        await test.f(runner)
        await runner.postTest(test);
        resolve();
      }, test.description);
    }).catch(_ => {});
  });
  Promise.all(allTests).then(_ => runner.teardown());
}
