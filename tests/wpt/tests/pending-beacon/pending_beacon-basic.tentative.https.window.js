// META: script=./resources/pending_beacon-helper.js

'use strict';

test(() => {
  assert_throws_js(TypeError, () => new PendingBeacon('/'));
}, `PendingBeacon cannot be constructed directly.`);

for (const beaconType of BeaconTypes) {
  test(() => {
    assert_throws_js(TypeError, () => new beaconType.type());
    assert_throws_js(TypeError, () => new beaconType.type(undefined));
    assert_throws_js(TypeError, () => new beaconType.type(null));
  }, `${beaconType.name}: constructor throws TypeError if URL is missing.`);

  test(() => {
    assert_throws_js(
        TypeError, () => new beaconType.type('http://www.google.com'));
    assert_throws_js(TypeError, () => new beaconType.type('file://tmp'));
    assert_throws_js(TypeError, () => new beaconType.type('ssh://example.com'));
    assert_throws_js(TypeError, () => new beaconType.type('wss://example.com'));
    assert_throws_js(TypeError, () => new beaconType.type('about:blank'));
    assert_throws_js(
        TypeError, () => new beaconType.type(`javascript:alert('');`));
  }, `${beaconType.name}: constructor throws TypeError on non-HTTPS URL.`);

  test(() => {
    const beacon = new beaconType.type('/');
    assert_equals(beacon.url, '/');
    assert_equals(beacon.method, beaconType.expectedMethod);
    assert_equals(beacon.backgroundTimeout, -1);
    assert_equals(beacon.timeout, -1);
    assert_true(beacon.pending);
  }, `${beaconType.name}: constructor sets default properties if missing.`);

  test(() => {
    const beacon = new beaconType.type(
        'https://www.google.com', {backgroundTimeout: 200, timeout: 100});
    assert_equals(beacon.url, 'https://www.google.com');
    assert_equals(beacon.method, beaconType.expectedMethod);
    assert_equals(beacon.backgroundTimeout, 200);
    assert_equals(beacon.timeout, 100);
    assert_true(beacon.pending);
  }, `${beaconType.name}: constructor can set properties.`);

  test(() => {
    let beacon = new beaconType.type(
        'https://www.google.com',
        {method: 'GET', backgroundTimeout: 200, timeout: 100});

    beacon.backgroundTimeout = 400;
    assert_equals(beacon.backgroundTimeout, 400);

    beacon.timeout = 600;
    assert_equals(beacon.timeout, 600);
  }, `${beaconType.name}: 'backgroundTimeout' & 'timeout' can be mutated.`);

  test(
      () => {
        let beacon = new beaconType.type('https://www.google.com');

        assert_throws_js(TypeError, () => beacon.url = '/');
        assert_throws_js(TypeError, () => beacon.method = 'FOO');
        assert_throws_js(TypeError, () => beacon.pending = false);
      },
      `${beaconType.name}: throws TypeError when mutating ` +
          `'url', 'method', 'pending'.`);
}

test(() => {
  let beacon = new PendingGetBeacon('/');

  assert_throws_js(TypeError, () => new beacon.setURL());
  assert_throws_js(TypeError, () => new beacon.setURL(undefined));
  assert_throws_js(TypeError, () => new beacon.setURL(null));
}, `PendingGetBeacon: setURL() throws TypeError if URL is missing.`);

test(() => {
  let beacon = new PendingGetBeacon('/');

  assert_throws_js(TypeError, () => beacon.setURL('http://www.google.com'));
  assert_throws_js(TypeError, () => beacon.setURL('file://tmp'));
  assert_throws_js(TypeError, () => beacon.setURL('ssh://example.com'));
  assert_throws_js(TypeError, () => beacon.setURL('wss://example.com'));
  assert_throws_js(TypeError, () => beacon.setURL('about:blank'));
  assert_throws_js(TypeError, () => beacon.setURL(`javascript:alert('');`));
}, `PendingGetBeacon: setURL() throws TypeError on non-HTTPS URL.`);
