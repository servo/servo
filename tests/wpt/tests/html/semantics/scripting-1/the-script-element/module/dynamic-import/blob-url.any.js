// META: global=window,dedicatedworker,sharedworker,dedicatedworker-module,sharedworker-module

function objectUrlFromModule(module) {
  const blob = new Blob([module], { type: "text/javascript" });
  return URL.createObjectURL(blob);
}

const moduleText = `export const foo = "bar";`;

promise_test(async (t) => {
  const moduleBlobUrl = objectUrlFromModule(moduleText);
  t.add_cleanup(() => URL.revokeObjectURL(moduleBlobUrl));

  const module = await import(moduleBlobUrl);
  assert_equals(module.foo, "bar");
}, "Blob URLs are supported in dynamic imports");

promise_test(async (t) => {
  const moduleBlobUrl = objectUrlFromModule(moduleText);
  t.add_cleanup(() => URL.revokeObjectURL(moduleBlobUrl));

  const module1 = await import(moduleBlobUrl);
  const module2 = await import(moduleBlobUrl);
  assert_equals(module1, module2);
}, "Identical blob URLs resolve to the same module");

promise_test(async (t) => {
  const moduleBlob = new Blob([moduleText], { type: "text/javascript" });
  const moduleBlobUrl1 = URL.createObjectURL(moduleBlob);
  const moduleBlobUrl2 = URL.createObjectURL(moduleBlob);
  t.add_cleanup(() => {
    URL.revokeObjectURL(moduleBlobUrl1);
    URL.revokeObjectURL(moduleBlobUrl2);
  });

  const module1 = await import(moduleBlobUrl1);
  const module2 = await import(moduleBlobUrl2);
  assert_not_equals(module1, module2);
}, "Different blob URLs pointing to the same blob resolve to different modules");

promise_test(async (t) => {
  const moduleBlobUrl = objectUrlFromModule(moduleText);
  URL.revokeObjectURL(moduleBlobUrl);

  await promise_rejects_js(t, TypeError, import(moduleBlobUrl));
}, "A revoked blob URL will not resolve");

promise_test(async () => {
  const moduleBlobUrl = objectUrlFromModule(moduleText);
  const module1 = await import(moduleBlobUrl);

  URL.revokeObjectURL(moduleBlobUrl);

  const module2 = await import(moduleBlobUrl);
  assert_equals(module1, module2);
}, "A revoked blob URL will resolve if it's already in the module graph");

promise_test(async () => {
  const moduleBlobUrl = objectUrlFromModule(moduleText);

  const importPromise = import(moduleBlobUrl);
  URL.revokeObjectURL(moduleBlobUrl);

  const module = await importPromise;
  assert_equals(module.foo, "bar");
}, "Revoking a blob URL immediately after calling import will not fail");
