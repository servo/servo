// META: title=Stopping a capture track and its clones
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=permission-helper.js

for (const kind of ["audio", "video"]) {
  promise_test(async t => {
    const permission = kind === "audio" ? "microphone" : "camera";
    await setMediaPermission("granted", [permission]);
    t.add_cleanup(() =>
      test_driver.set_permission({name: permission}, "prompt"));
    const stream = await navigator.mediaDevices.getUserMedia({[kind]: true});
    t.add_cleanup(() => stream.getTracks().forEach(track => track.stop()));
    const [track] = stream.getTracks();
    const clone = track.clone();
    t.add_cleanup(() => clone.stop());
    track.addEventListener("ended", t.unreached_func(
      "Calling stop() must not dispatch an ended event"));
    assert_equals(track.readyState, "live");
    assert_equals(clone.readyState, "live");

    track.stop();
    assert_equals(track.readyState, "ended");
    assert_equals(clone.readyState, "live",
      "Stopping one track does not end its live clone");
    track.stop();
    assert_equals(clone.readyState, "live");

    const endedClone = track.clone();
    t.add_cleanup(() => endedClone.stop());
    assert_equals(endedClone.readyState, "ended",
      "Cloning a stopped track preserves its state");
    clone.stop();
    clone.stop();
    assert_equals(clone.readyState, "ended");
    await new Promise(resolve => t.step_timeout(resolve, 0));
  }, `Stopping ${kind} capture tracks is idempotent and preserves clone state`);
}
