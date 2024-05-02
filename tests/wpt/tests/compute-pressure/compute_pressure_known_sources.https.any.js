// META: global=window,dedicatedworker,sharedworker

'use strict';

test(() => {
  // Compute Pressure should support at least "cpu"
  const sources = PressureObserver.knownSources;
  assert_in_array('cpu', sources);
}, 'PressureObserver should support at least "cpu"');

test(() => {
  // Compute Pressure should be frozen array
  const sources = PressureObserver.knownSources;
  assert_equals(sources, PressureObserver.knownSources);
}, 'PressureObserver must return always the same array');

test(() => {
  // Compute Pressure should be frozen array
  let sources = PressureObserver.knownSources;
  assert_equals(Object.isFrozen(sources), true);
}, 'PressureObserver must return a frozen array');
