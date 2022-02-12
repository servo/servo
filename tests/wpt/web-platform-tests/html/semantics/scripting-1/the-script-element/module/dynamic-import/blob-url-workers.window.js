function objectUrlFromModule(module) {
  const blob = new Blob([module], { type: "text/javascript" });
  return URL.createObjectURL(blob);
}

const moduleText = `export const foo = "bar";`;

async_test((t) => {
  const moduleBlobUrl = objectUrlFromModule(moduleText);
  t.add_cleanup(() => URL.revokeObjectURL(moduleBlobUrl));

  const worker = new Worker("./resources/blob-url-worker.js");
  worker.postMessage(moduleBlobUrl);

  worker.addEventListener(
    "message",
    t.step_func_done((evt) => {
      assert_true(evt.data.importSucceeded);
      assert_equals(evt.data.module.foo, "bar");
    })
  );
}, "A blob URL created in a window agent can be imported from a worker");

async_test((t) => {
  const moduleBlobUrl = objectUrlFromModule(moduleText);
  URL.revokeObjectURL(moduleBlobUrl);

  const worker = new Worker("./resources/blob-url-worker.js");
  worker.postMessage(moduleBlobUrl);

  worker.addEventListener(
    "message",
    t.step_func_done((evt) => {
      assert_false(evt.data.importSucceeded);
      assert_equals(evt.data.errorName, "TypeError");
    })
  );
}, "A blob URL revoked in a window agent will not resolve in a worker");

promise_test(async (t) => {
  const moduleBlobUrl = objectUrlFromModule(moduleText);

  await import(moduleBlobUrl);

  URL.revokeObjectURL(moduleBlobUrl);

  const worker = new Worker("./resources/blob-url-worker.js");
  worker.postMessage(moduleBlobUrl);

  await new Promise((resolve) => {
    worker.addEventListener(
      "message",
      t.step_func((evt) => {
        assert_false(evt.data.importSucceeded);
        assert_equals(evt.data.errorName, "TypeError");
        resolve();
      })
    );
  });
}, "A revoked blob URL will not resolve in a worker even if it's in the window's module graph");
