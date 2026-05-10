importScripts("{{location[server]}}/resources/testharness.js");

async_test(function(t) {
  const observer = new ReportingObserver(t.step_func_done((reports) => {
    done();
  }));

  observer.observe();
  const off = new OffscreenCanvas(1, 1);
  const ctx = off.getContext("2d");
  ctx.fillRect(0, 0, 1, 1);
  // Trigger deprecation
  off.toBlob().then(() => {});
}, "Worker should trigger a deprecation report");
