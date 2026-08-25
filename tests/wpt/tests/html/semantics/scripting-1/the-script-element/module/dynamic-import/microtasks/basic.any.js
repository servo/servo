// META: global=window,dedicatedworker,sharedworker
// META: script=ticker.js

promise_test(async t => {
  const getCount = ticker(1000);

  const importP = import("<invalid>");
  await promise_rejects_js(t, TypeError, importP, 'import() should reject');

  assert_less_than(getCount(), 1000);
}, "import() should not drain the microtask queue if it fails during specifier resolution");

promise_test(async t => {
  // Use Date.now() to ensure that the module is not in the module map
  const specifier = "./empty-module.js?" + Date.now();

  await import(specifier);

  const getCount = ticker(1000);
  await import(specifier);
  assert_less_than(getCount(), 1000);
}, "import() should not drain the microtask queue when loading an already loaded module");

promise_test(async t => {
  // Use Date.now() to ensure that the module is not in the module map
  const specifier = "./empty-module.js?" + Date.now();

  const getCount = ticker(1e6);
  await import(specifier);
  assert_equals(getCount(), 1e6);
}, "import() should drain the microtask queue when fetching a new module");

