'use strict';

test(() => {
  // Compute Pressure should support at least "cpu"
  const sources = ComputePressureObserver.supportedSources;
  assert_in_array('cpu', sources);
}, 'ComputePressureObserver should support at least "cpu"');

test(() => {
  // Compute Pressure should be frozen array
  const sources = ComputePressureObserver.supportedSources;
  assert_equals(sources, ComputePressureObserver.supportedSources);
}, 'ComputePressureObserver must return always the same array');

test(() => {
  // Compute Pressure should be frozen array
  let sources = ComputePressureObserver.supportedSources;
  assert_equals(Object.isFrozen(), true);
}, 'ComputePressureObserver must return a frozen array');
