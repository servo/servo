// META: script=./resources/pending_beacon-helper.js

'use strict';

for (const beaconType of BeaconTypes) {
  test(() => {
    const beacon = new beaconType.type('https://www.google.com');
    assert_true(beacon.pending);
    beacon.deactivate();
    assert_false(beacon.pending);
  }, `${beaconType.name}: deactivate() changes 'pending' state.`);
}
