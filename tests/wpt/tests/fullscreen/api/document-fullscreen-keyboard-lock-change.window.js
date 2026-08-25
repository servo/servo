// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-actions.js
// META: script=/resources/testdriver-vendor.js
// META: timeout=long

promise_test(async t => {
  t.add_cleanup(() => document.exitFullscreen().catch(() => {}));

  let { promise: fullscreenEnterPromise, resolve } = Promise.withResolvers();
  document.addEventListener("fullscreenchange", resolve, { once: true });

  await test_driver.bless("requestFullscreen", () => document.body.requestFullscreen({ keyboardLock: "browser" }));
  await fullscreenEnterPromise;
  assert_equals(document.fullscreenElement, document.body, "fullscreen should activate");

  await test_driver.send_keys(document.body, '\uE00C');
  await new Promise(r => t.step_timeout(r, 2000));
  assert_equals(document.fullscreenElement, document.body, "fullscreen should stay");

  document.onfullscreenchange = t.unreached_func("No extra fullscreen change is expected by option change");
  await test_driver.bless("requestFullscreen", () => document.body.requestFullscreen());
  await new Promise(requestAnimationFrame);
  document.onfullscreenchange = null;

  await test_driver.send_keys(document.body, '\uE00C');
  await new Promise(r => t.step_timeout(r, 2000));
  assert_equals(document.fullscreenElement, null, "fullscreen should deactivate");
}, `Requesting fullscreen again without keyboard lock should disable it`);

// TODO(krosylight): we should be able to test the reverse way, but there's no good way to do so.
