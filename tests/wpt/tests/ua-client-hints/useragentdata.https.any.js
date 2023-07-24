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
  return navigator.userAgentData.getHighEntropyValues(["architecture"]).then(
    hints => assert_true(["x86", "arm"].some(item => item == hints.architecture))
  );
}, "Arch should be one of two permitted values.");
