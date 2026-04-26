// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-actions.js
// META: script=/resources/testdriver-vendor.js
// META: timeout=long

async function requestFullscreen(element, options) {
  let { promise: fullscreenEnterPromise, resolve } = Promise.withResolvers();
  document.addEventListener("fullscreenchange", resolve, { once: true });
  // We could not use bless() or test_driver.click() because,
  // 1. bless() appends its button to document.body, which sits behind the
  //    fullscreen backdrop when a shadow DOM element is already fullscreen.
  // 2. test_driver.click() on shadow DOM elements fails the
  //    element-click-intercepted check.
  await new test_driver.Actions()
    .pointerMove(0, 0, {origin: document.documentElement})
    .pointerDown()
    .pointerUp()
    .send();
  await element.requestFullscreen(options);
  await fullscreenEnterPromise;
}

async function exitFullscreen() {
  let { promise: fullscreenExitPromise, resolve } = Promise.withResolvers();
  document.addEventListener("fullscreenchange", resolve, { once: true });
  await document.exitFullscreen();
  await fullscreenExitPromise;
}

async function holdEscapeKey() {
  // Press Escape for 5 seconds
  // Holding the key makes it repeat, so do the same here
  let actions = new test_driver.Actions()
    .keyDown("\uE00C")
    .addTick(1000)
    .keyDown("\uE00C")
    .addTick(1000)
    .keyDown("\uE00C")
    .addTick(1000)
    .keyDown("\uE00C")
    .addTick(1000)
    .keyDown("\uE00C")
    .addTick(1000)
    .keyDown("\uE00C")
    .keyUp("\uE00C")
  await actions.send();
}

promise_test(async t => {
  t.add_cleanup(() => document.exitFullscreen().catch(() => {}));

  // Setup shadow DOM.
  let host = document.createElement("div");
  document.body.appendChild(host);

  let shadowRoot = host.attachShadow({ mode: "open" });
  let outer = document.createElement("div");
  outer.textContent = "Outer";
  shadowRoot.appendChild(outer);

  let inner = document.createElement("div");
  inner.textContent = "Inner";
  outer.appendChild(inner);

  // Request fullscreen on the outer node in shadow DOM with keyboard lock.
  await requestFullscreen(outer, { keyboardLock: "browser" });
  assert_equals(document.fullscreenElement, host, "check document.fullscreen");
  assert_equals(shadowRoot.fullscreenElement, outer, "check shadowRoot.fullscreen");

  // Request fullscreen on the inner node in shadow DOM without keyboard lock.
  await requestFullscreen(inner, { keyboardLock: "none" });
  assert_equals(document.fullscreenElement, host, "check document.fullscreen");
  assert_equals(shadowRoot.fullscreenElement, inner, "check shadowRoot.fullscreen");

  // exitfullscreen should back to previous keyboard lock state.
  await exitFullscreen();
  assert_equals(document.fullscreenElement, host, "check document.fullscreen");
  assert_equals(shadowRoot.fullscreenElement, outer, "check shadowRoot.fullscreen");

  // Pressing esc should not exit fullscreen.
  await test_driver.send_keys(document.body, '\uE00C');
  await new Promise(r => t.step_timeout(r, 2000));
  assert_equals(document.fullscreenElement, host, "check document.fullscreen");
  assert_equals(shadowRoot.fullscreenElement, outer, "check shadowRoot.fullscreen");

  // Long pressing esc should exit fullscreen.
  await holdEscapeKey();
  assert_equals(document.fullscreenElement, null, "fullscreen should deactivate");
}, `Requesting and exiting fullscreen with different keyboard lock options in Shadow DOM should work as expected`);
