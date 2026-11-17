// META: title=getUserMedia requires every requested permission
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

for (const denied of ["camera", "microphone"]) {
  promise_test(async t => {
    for (const name of ["camera", "microphone"]) {
      await test_driver.set_permission(
        {name}, name === denied ? "denied" : "granted");
      t.add_cleanup(() => test_driver.set_permission({name}, "prompt"));
    }
    await promise_rejects_dom(
      t, "NotAllowedError",
      navigator.mediaDevices.getUserMedia({audio: true, video: true}));
  }, `A denied ${denied} permission rejects a combined capture request`);
}
