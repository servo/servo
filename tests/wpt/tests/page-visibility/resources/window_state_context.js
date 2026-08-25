function window_state_context(t) {
  let rect = null;
  let state = "restored";
  t.add_cleanup(async () => {
    if (state === "minimized") {
      await restore();
    }
  });
  async function restore() {
    if (state !== "minimized") {
      return;
    }
    state = "restoring";
    await test_driver.set_window_rect(rect);
    state = "restored";
  }

  async function minimize() {
    state = "minimized";
    rect = await test_driver.minimize_window();
  }

  function visibilityEventPromise() {
    return new Promise((resolve) =>
      new PerformanceObserver((entries, observer) => {
        observer.disconnect();
        resolve();
      }).observe({ type: "visibility-state" })
    );
  }

  async function minimizeAndWait() {
    const promise = visibilityEventPromise();
    await Promise.all([minimize(), promise]);
    await new Promise((resolve) => t.step_timeout(resolve, 0));
  }

  async function restoreAndWait() {
    const promise = visibilityEventPromise();
    await Promise.all([restore(), promise]);
    await new Promise((resolve) => t.step_timeout(resolve, 0));
  }

  return { minimize, restore, minimizeAndWait, restoreAndWait };
}
