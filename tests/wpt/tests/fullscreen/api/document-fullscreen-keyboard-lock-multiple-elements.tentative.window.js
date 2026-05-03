// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-actions.js
// META: script=/resources/testdriver-vendor.js
// META: timeout=long

promise_test(async t => {
  t.add_cleanup(() => document.exitFullscreen().catch(() => {}));

  await test_driver.bless("requestFullscreen", () => document.body.requestFullscreen({ keyboardLock: "browser" }));
  await new Promise(resolve => { document.onfullscreenchange = resolve; });
  assert_equals(document.fullscreenElement, document.body, "fullscreen should activate");

  await test_driver.send_keys(document.body, '\uE00C');
  await new Promise(r => t.step_timeout(r, 2000));
  assert_equals(document.fullscreenElement, document.body, "fullscreen should stay");

  const div = document.createElement('div');
  document.body.append(div);

  await test_driver.bless("requestFullscreen", () => div.requestFullscreen());
  await new Promise(resolve => { document.onfullscreenchange = resolve; });

  assert_equals(document.fullscreenElement, div, "div should be the fullscreenElement");

  await document.exitFullscreen();
  await new Promise(resolve => { document.onfullscreenchange = resolve; });

  await test_driver.send_keys(document.body, '\uE00C');
  await new Promise(r => t.step_timeout(r, 2000));
  assert_equals(document.fullscreenElement, document.body, "fullscreen should stay");

  await document.exitFullscreen();
}, `Requesting fullscreen with keyboard lock, then on another element without keyboard lock, then exitFullscreen()`);
