const notification_args = [
  "Radio check",
  {
      dir: "ltr",
      lang: "aa",
      body: "This is a radio check.",
      tag: "radio_check999",
      icon: `${location.origin}/icon.png`,
      requireInteraction: true,
      silent: true,
      data: fakeCustomData,
      actions: [{ action: "foo", title: "bar" }]
  }
];

// promise_tests because we need to wait for promise_setup
function notification_instance_test(createFn, testTitle) {
  let n;
  promise_test(async t => {
    n = await createFn(t);
  }, `${testTitle}: Setup`);
  promise_test(async () => {
    assert_equals(n.title, "Radio check")
  }, `${testTitle}: Attribute exists with expected value: title`)
  promise_test(async () => {
    assert_equals(n.dir, "ltr")
  }, `${testTitle}: Attribute exists with expected value: dir`)
  promise_test(async () => {
    assert_equals(n.lang, "aa")
  }, `${testTitle}: Attribute exists with expected value: lang`)
  promise_test(async () => {
    assert_equals(n.body, "This is a radio check.")
  }, `${testTitle}: Attribute exists with expected value: body`)
  promise_test(async () => {
    assert_equals(n.tag, "radio_check999")
  }, `${testTitle}: Attribute exists with expected value: tag`)
  promise_test(async () => {
    assert_equals(n.icon, `${location.origin}/icon.png`)
  }, `${testTitle}: Attribute exists with expected value: icon`)
  promise_test(async () => {
    assert_true(n.requireInteraction);
  }, `${testTitle}: Attribute exists with expected value: requireInteraction`)
  promise_test(async () => {
    assert_true(n.silent);
  }, `${testTitle}: Attribute exists with expected value: silent`)
  promise_test(async () => {
    assert_custom_data(n.data);
  }, `${testTitle}: Attribute exists with expected value: data`)
  promise_test(async () => {
    for (const [i, action] of n.actions.entries()) {
      assert_object_equals(action, notification_args[1].actions[i]);
    }
  }, `${testTitle}: Attribute exists with expected value: actions`)
}
