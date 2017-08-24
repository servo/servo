let expectFail = function (testName, specs, type, message) {
  for (let spec of specs) {
    test(function () {
      if (!spec.name) {
        spec.name = "--test-property";
      }
      let caught = null;
      try {
        CSS.registerProperty(spec);
      } catch (e) {
        caught = e;
      }
      let passed = caught instanceof DOMException && caught.name == type;
      try {
        assert_true(passed, message + " " + JSON.stringify(spec));
      } finally {
        if (!passed) {
          CSS.unregisterProperty(spec.name);
        }
      }
    }, testName + ": " + JSON.stringify(spec));
  }
};

let expectSucceed = function (testName, specs, message) {
  for (let spec of specs) {
    test(function () {
      if (!spec.name) {
        spec.name = "--test-property";
      }
      let caught = null;
      try {
        CSS.registerProperty(spec);
      } catch (e) {
        caught = e;
      }
      let passed = !caught;
      assert_true(passed, message + " " + JSON.stringify(spec));
      if (passed) {
        CSS.unregisterProperty(spec.name);
      }
    }, testName + ": " + JSON.stringify(spec));
  }
};

