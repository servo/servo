// META: title=tests for navigator.userAgentData

test(t => {
  const brands = navigator.userAgentData.brands;
  assert_true(brands.every(brand => brand.brand.length < 32),
    "No brand should be longer than 32 characters.");
});

test(t => {
  const uaData = navigator.userAgentData.toJSON();
  assert_own_property(uaData, "brands", "toJSON() output has brands member");
  assert_own_property(uaData, "mobile", "toJSON() output has mobile member");
  assert_own_property(uaData, "platform", "toJSON() output has platform member");
}, "test NavigatorUAData.toJSON() output");

promise_test(() => {
  return navigator.userAgentData.getHighEntropyValues([]).then(hints => {
    assert_own_property(hints, "brands", "brands is returned by default");
    assert_own_property(hints, "mobile", "mobile is returned by default");
    assert_own_property(hints, "platform", "platform is returned by default");
  });
}, "getHighEntropyValues() should return low-entropy hints by default (1).");

promise_test(() => {
  return navigator.userAgentData.getHighEntropyValues(["wow64"]).then(hints => {
    assert_own_property(
        hints, "wow64", "requested high-entropy hint is returned");
    assert_own_property(hints, "brands", "brands is returned by default");
    assert_own_property(hints, "mobile", "mobile is returned by default");
    assert_own_property(hints, "platform", "platform is returned by default");
  });
}, "getHighEntropyValues() should return low-entropy hints by default (2).");

promise_test(() => {
  return navigator.userAgentData.getHighEntropyValues(["brands", "mobile"])
      .then(hints => {
        assert_own_property(hints, "brands", "requested brands is returned");
        assert_own_property(
            hints, "mobile", "requested mobile is returned by default");
        assert_own_property(
            hints, "platform", "platform is returned by default");
      });
}, "getHighEntropyValues() should return low-entropy hints by default (3).");

promise_test(() => {
  return navigator.userAgentData.getHighEntropyValues(["platform", "wow64"])
      .then(hints => {
        assert_own_property(hints, "brands", "brands is returned by default");
        assert_own_property(hints, "mobile", "mobile is returned by default");
        assert_own_property(
            hints, "platform", "requested platform is returned");
        assert_own_property(hints, "wow64", "requested wow64 is returned");
      });
}, "getHighEntropyValues() should return low-entropy hints by default (4).");

promise_test(() => {
  return navigator.userAgentData.getHighEntropyValues(["architecture"]).then(
    hints => assert_true(["x86", "arm"].some(item => item == hints.architecture))
  );
}, "Arch should be one of two permitted values.");
