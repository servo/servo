// META: title=Bare numeric capture constraints are preferences
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=permission-helper.js

for (const constraint of [{width: 1000000000}, {frameRate: 1000000000}]) {
  promise_test(async t => {
    await setMediaPermission("granted", ["camera"]);
    t.add_cleanup(() =>
      test_driver.set_permission({name: "camera"}, "prompt"));
    const stream = await navigator.mediaDevices.getUserMedia({video: constraint});
    t.add_cleanup(() => stream.getTracks().forEach(track => track.stop()));
    assert_equals(stream.getVideoTracks().length, 1);
  }, `An unsatisfied bare ${Object.keys(constraint)[0]} preference does not reject capture`);
}
