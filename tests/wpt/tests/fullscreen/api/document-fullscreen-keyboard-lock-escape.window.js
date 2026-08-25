// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-actions.js
// META: script=/resources/testdriver-vendor.js
// META: timeout=long

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

for (const preventDefault of [true, false]) {
  const withEv = preventDefault ? "with" : "without";

  promise_test(async t => {
    t.add_cleanup(() => document.exitFullscreen().catch(() => {}));

    const signal = t.get_signal();
    await test_driver.bless("requestFullscreen", () => document.body.requestFullscreen({ keyboardLock: "browser" }));
    assert_equals(document.fullscreenElement, document.body, "fullscreen should activate");

    let { promise: fullscreenExitPromise, resolve } = Promise.withResolvers();
    document.addEventListener("fullscreenchange", resolve, { once: true });

    if (preventDefault) {
      addEventListener("keydown", ev => ev.preventDefault(), { signal });
    }

    await holdEscapeKey();
    await fullscreenExitPromise;
    assert_equals(document.fullscreenElement, null, "fullscreen should deactivate");
  }, `Holding Escape ${withEv} event.preventDefault() should cause fullscreen exit`);

  promise_test(async t => {
    t.add_cleanup(() => document.exitFullscreen().catch(() => {}));

    const signal = t.get_signal();
    await test_driver.bless("requestFullscreen", () => document.body.requestFullscreen({ keyboardLock: "browser" }));
    assert_equals(document.fullscreenElement, document.body, "fullscreen should activate");

    if (preventDefault) {
      addEventListener("keydown", ev => ev.preventDefault(), { signal });
    }
    await test_driver.send_keys(document.body, '\uE00C');
    await new Promise(r => t.step_timeout(r, 2000));
    assert_equals(document.fullscreenElement, document.body, "fullscreen should stay");
  }, `Tapping Escape ${withEv} event.preventDefault() should not cause fullscreen exit`);
}
