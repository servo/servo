'use strict';

test(() => {
  // Compute Pressure should support at least "cpu"
  const sources = PressureObserver.supportedSources;
  assert_in_array('cpu', sources);
}, 'PressureObserver should support at least "cpu"');

test(() => {
  // Compute Pressure should be frozen array
  const sources = PressureObserver.supportedSources;
  assert_equals(sources, PressureObserver.supportedSources);
}, 'PressureObserver must return always the same array');

test(() => {
  // Compute Pressure should be frozen array
  let sources = PressureObserver.supportedSources;
  assert_equals(Object.isFrozen(), true);
}, 'PressureObserver must return a frozen array');
