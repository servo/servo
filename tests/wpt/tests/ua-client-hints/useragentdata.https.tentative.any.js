// META: title=tests for navigator.userAgentData on Linux

promise_test(() => {
  return navigator.userAgentData.getHighEntropyValues(["platformVersion", "wow64"]).then(
    hints => {
      if (navigator.userAgentData.platform === "Linux") {
        assert_true(hints.platformVersion === "");
        assert_equals(hints.wow64, false);
      }
    }
  );
}, "Platform version and wow64-ness on Linux should be fixed values");
