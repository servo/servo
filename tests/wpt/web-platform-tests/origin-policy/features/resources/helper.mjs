"use strict";

export function runFPTest({ camera, geolocation }) {
  test(() => {
    assert_equals(document.featurePolicy.allowsFeature('camera', 'https://example.com/'), camera, 'camera');
    assert_equals(document.featurePolicy.allowsFeature('geolocation', 'https://example.com/'), geolocation, 'geolocation');
  });
}
