const feature_policies = {
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
  "/feature-policy/resources/feature-policy-generic-sensor.html#";
const cross_origin_src =
  "https://{{domains[www]}}:{{ports[https][0]}}" + same_origin_src;
const base_src = "/feature-policy/resources/redirect-on-load.html#";

function get_feature_policies_for_sensor(sensorType) {
  return feature_policies[sensorType];
}

function run_fp_tests_disabled(sensorName) {
  const sensorType = self[sensorName];
  const featureNameList = feature_policies[sensorName];
  const header = "Feature-Policy header " + featureNameList.join(" 'none';") + " 'none'";
  const desc = "'new " + sensorName + "()'";

  test(() => {
    assert_true(sensorName in self);
    assert_throws_dom("SecurityError", () => {new sensorType()});
  }, `${sensorName}: ${header} disallows the top-level document.`);

  async_test(t => {
    assert_true(sensorName in self);
    test_feature_availability(
      desc,
      t,
      same_origin_src + sensorName,
      expect_feature_unavailable_default
    );
  }, `${sensorName}: ${header} disallows same-origin iframes.`);

  async_test(t => {
    assert_true(sensorName in self);
    test_feature_availability(
      desc,
      t,
      cross_origin_src + sensorName,
      expect_feature_unavailable_default
    );
  }, `${sensorName}: ${header} disallows cross-origin iframes.`);
}

function run_fp_tests_enabled(sensorName) {
  const featureNameList = feature_policies[sensorName];
  const header = "Feature-Policy header " + featureNameList.join(" *;") + " *";
  const desc = "'new " + sensorName + "()'";

  test(() => {
    assert_true(sensorName in self);
  }, `${sensorName}: ${header} allows the top-level document.`);

  async_test(t => {
    assert_true(sensorName in self);
    test_feature_availability(
      desc,
      t,
      same_origin_src + sensorName,
      expect_feature_available_default
    );
  }, `${sensorName}: ${header} allows same-origin iframes.`);

  async_test(t => {
    assert_true(sensorName in self);
    test_feature_availability(
      desc,
      t,
      cross_origin_src + sensorName,
      expect_feature_available_default
    );
  }, `${sensorName}: ${header} allows cross-origin iframes.`);
}

function run_fp_tests_enabled_by_attribute(sensorName) {
  const featureNameList = feature_policies[sensorName];
  const header = "Feature-Policy allow='" + featureNameList.join(" ") + "' attribute";
  const desc = "'new " + sensorName + "()'";

  async_test(t => {
    assert_true(sensorName in self);
    test_feature_availability(
      desc,
      t,
      same_origin_src + sensorName,
      expect_feature_available_default,
      featureNameList.join(";")
    );
  }, `${sensorName}: ${header} allows same-origin iframe`);

  async_test(t => {
    assert_true(sensorName in self);
    test_feature_availability(
      desc,
      t,
      cross_origin_src + sensorName,
      expect_feature_available_default,
      featureNameList.join(";")
    );
  }, `${sensorName}: ${header} allows cross-origin iframe`);
}

function run_fp_tests_enabled_by_attribute_redirect_on_load(sensorName) {
  const featureNameList = feature_policies[sensorName];
  const header = "Feature-Policy allow='" + featureNameList.join(" ") + "' attribute";
  const desc = "'new " + sensorName + "()'";

  async_test(t => {
    assert_true(sensorName in self);
    test_feature_availability(
      desc,
      t,
      base_src + same_origin_src + sensorName,
      expect_feature_available_default,
      featureNameList.join(";")
    );
  }, `${sensorName}: ${header} allows same-origin relocation`);

  async_test(t => {
    assert_true(sensorName in self);
    test_feature_availability(
      desc,
      t,
      base_src + cross_origin_src + sensorName,
      expect_feature_unavailable_default,
      featureNameList.join(";")
    );
  }, `${sensorName}: ${header} disallows cross-origin relocation`);
}

function run_fp_tests_enabled_on_self_origin(sensorName) {
  const featureNameList = feature_policies[sensorName];
  const header = "Feature-Policy header " + featureNameList.join(" 'self';") + " 'self'";
  const desc = "'new " + sensorName + "()'";

  test(() => {
    assert_true(sensorName in self);
  }, `${sensorName}: ${header} allows the top-level document.`);

  async_test(t => {
    assert_true(sensorName in self);
    test_feature_availability(
      desc,
      t,
      same_origin_src + sensorName,
      expect_feature_available_default
    );
  }, `${sensorName}: ${header} allows same-origin iframes.`);

  async_test(t => {
    assert_true(sensorName in self);
    test_feature_availability(
      desc,
      t,
      cross_origin_src + sensorName,
      expect_feature_unavailable_default
    );
  }, `${sensorName}: ${header} disallows cross-origin iframes.`);
}
