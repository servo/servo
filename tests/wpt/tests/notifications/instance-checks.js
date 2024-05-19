const notification_args = [
  "Radio check",
  {
      dir: "ltr",
      lang: "aa",
      body: "This is a radio check.",
      tag: "radio_check999",
      icon: `${location.origin}/icon.png`,
      data: fakeCustomData,
  }
];

// promise_tests because we need to wait for promise_setup
function notification_instance_test(createFn, testTitle) {
  let n;
  promise_test(async t => {
    n = await createFn(t);
  }, `${testTitle}: Setup`);
  promise_test(async () => {
    assert_equals("Radio check", n.title)
  }, `${testTitle}: Attribute exists with expected value: title`)
  promise_test(async () => {
    assert_equals("ltr", n.dir)
  }, `${testTitle}: Attribute exists with expected value: dir`)
  promise_test(async () => {
    assert_equals("aa", n.lang)
  }, `${testTitle}: Attribute exists with expected value: lang`)
  promise_test(async () => {
    assert_equals("This is a radio check.", n.body)
  }, `${testTitle}: Attribute exists with expected value: body`)
  promise_test(async () => {
    assert_equals("radio_check999", n.tag)
  }, `${testTitle}: Attribute exists with expected value: tag`)
  promise_test(async () => {
    assert_equals(`${location.origin}/icon.png`, n.icon)
  }, `${testTitle}: Attribute exists with expected value: icon`)
  promise_test(async () => {
    assert_custom_data(n.data);
  }, `${testTitle}: Attribute exists with expected value: data`)
}
