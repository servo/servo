"use strict";

export function runFPTest({ camera, geolocation }) {
  test(() => {
    assert_equals(document.featurePolicy.allowsFeature('camera'), camera, 'camera');
    assert_equals(document.featurePolicy.allowsFeature('geolocation'), geolocation, 'geolocation');
  });
}
