const permissions_policies = {
  "AmbientLightSensor" : ["ambient-light-sensor"],
  "Accelerometer" : ["accelerometer"],
  "LinearAccelerationSensor" : ["accelerometer"],
  "GravitySensor" : ["accelerometer"],
  "Gyroscope" : ["gyroscope"],
  "GeolocationSensor" : ["geolocation"],
  "Magnetometer" : ["magnetometer"],
  "UncalibratedMagnetometer" : ["magnetometer"],
  "AbsoluteOrientationSensor" : ["accelerometer", "gyroscope", "magnetometer"],
  "RelativeOrientationSensor" : ["accelerometer", "gyroscope"]
};

const same_origin_src =
  "/permissions-policy/resources/permissions-policy-generic-sensor.html#";
const cross_origin_src =
  "https://{{domains[www]}}:{{ports[https][0]}}" + same_origin_src;
const base_src = "/permissions-policy/resources/redirect-on-load.html#";

function get_permissions_policies_for_sensor(sensorType) {
  return permissions_policies[sensorType];
}

function run_permissions_policy_tests_disabled(sensorName) {
  const sensorType = self[sensorName];
  const featureNameList = permissions_policies[sensorName];
  const header = "Permissions-Policy header " + featureNameList.join("=();") + "=()";
  const desc = "'new " + sensorName + "()'";

  test(() => {
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
    assert_throws_dom("SecurityError", () => {new sensorType()});
  }, `${sensorName}: ${header} disallows the top-level document.`);

  async_test(t => {
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
    test_feature_availability(
      desc,
      t,
      same_origin_src + sensorName,
      expect_feature_unavailable_default
    );
  }, `${sensorName}: ${header} disallows same-origin iframes.`);

  async_test(t => {
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
    test_feature_availability(
      desc,
      t,
      cross_origin_src + sensorName,
      expect_feature_unavailable_default
    );
  }, `${sensorName}: ${header} disallows cross-origin iframes.`);
}

function run_permissions_policy_tests_enabled(sensorName) {
  const featureNameList = permissions_policies[sensorName];
  const header = "Permissions-Policy header " + featureNameList.join("=*;") + "=*";
  const desc = "'new " + sensorName + "()'";

  test(() => {
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
  }, `${sensorName}: ${header} allows the top-level document.`);

  async_test(t => {
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
    test_feature_availability(
      desc,
      t,
      same_origin_src + sensorName,
      expect_feature_available_default
    );
  }, `${sensorName}: ${header} allows same-origin iframes.`);

  // Set allow="feature;feature;..." on iframe element to delegate features
  // under test to cross origin subframe.
  async_test(t => {
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
    test_feature_availability(
      desc,
      t,
      cross_origin_src + sensorName,
      expect_feature_available_default,
      permissions_policies[sensorName].join(";")
    );
  }, `${sensorName}: ${header} allows cross-origin iframes.`);
}

function run_permissions_policy_tests_enabled_by_attribute(sensorName) {
  const featureNameList = permissions_policies[sensorName];
  const header = "Permissions-Policy allow='" + featureNameList.join(" ") + "' attribute";
  const desc = "'new " + sensorName + "()'";

  async_test(t => {
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
    test_feature_availability(
      desc,
      t,
      same_origin_src + sensorName,
      expect_feature_available_default,
      featureNameList.join(";")
    );
  }, `${sensorName}: ${header} allows same-origin iframe`);

  async_test(t => {
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
    test_feature_availability(
      desc,
      t,
      cross_origin_src + sensorName,
      expect_feature_available_default,
      featureNameList.join(";")
    );
  }, `${sensorName}: ${header} allows cross-origin iframe`);
}

function run_permissions_policy_tests_enabled_by_attribute_redirect_on_load(sensorName) {
  const featureNameList = permissions_policies[sensorName];
  const header = "Permissions-Policy allow='" + featureNameList.join(" ") + "' attribute";
  const desc = "'new " + sensorName + "()'";

  async_test(t => {
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
    test_feature_availability(
      desc,
      t,
      base_src + same_origin_src + sensorName,
      expect_feature_available_default,
      featureNameList.join(";")
    );
  }, `${sensorName}: ${header} allows same-origin relocation`);

  async_test(t => {
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
    test_feature_availability(
      desc,
      t,
      base_src + cross_origin_src + sensorName,
      expect_feature_unavailable_default,
      featureNameList.join(";")
    );
  }, `${sensorName}: ${header} disallows cross-origin relocation`);
}

function run_permissions_policy_tests_enabled_on_self_origin(sensorName) {
  const featureNameList = permissions_policies[sensorName];
  const header = "Permissions-Policy header " + featureNameList.join("=(self);") + "=(self)";
  const desc = "'new " + sensorName + "()'";

  test(() => {
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
  }, `${sensorName}: ${header} allows the top-level document.`);

  async_test(t => {
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
    test_feature_availability(
      desc,
      t,
      same_origin_src + sensorName,
      expect_feature_available_default
    );
  }, `${sensorName}: ${header} allows same-origin iframes.`);

  async_test(t => {
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
    test_feature_availability(
      desc,
      t,
      cross_origin_src + sensorName,
      expect_feature_unavailable_default
    );
  }, `${sensorName}: ${header} disallows cross-origin iframes.`);
}

// Backward compatibility aliases for legacy function names
// TODO: Remove these once all test files are updated to use the new names
const feature_policies = permissions_policies;
const get_feature_policies_for_sensor = get_permissions_policies_for_sensor;
const run_fp_tests_disabled = run_permissions_policy_tests_disabled;
const run_fp_tests_enabled = run_permissions_policy_tests_enabled;
const run_fp_tests_enabled_by_attribute = run_permissions_policy_tests_enabled_by_attribute;
const run_fp_tests_enabled_by_attribute_redirect_on_load = run_permissions_policy_tests_enabled_by_attribute_redirect_on_load;
const run_fp_tests_enabled_on_self_origin = run_permissions_policy_tests_enabled_on_self_origin;
