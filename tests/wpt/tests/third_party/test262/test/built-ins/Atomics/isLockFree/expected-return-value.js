// Copyright (C) 2017 Mozilla Corporation.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.islockfree
description: >
  Atomics.isLockFree( size )
    Let n be ? ToInteger(size).
    Let AR be the Agent Record of the surrounding agent.
    If n equals 1, return AR.[[IsLockFree1]].
    If n equals 2, return AR.[[IsLockFree2]].
    If n equals 4, return true.
    If n equals 8, return AR.[[IsLockFree8]].
    Return false.
features: [Atomics, Array.prototype.includes]
---*/

// These are the only counts that we care about tracking.
var isLockFree1;
var isLockFree2;
var isLockFree8;

{
  isLockFree1 = Atomics.isLockFree(1);
  //
  // If n equals 1, return AR.[[IsLockFree1]].
  //
  assert.sameValue(typeof isLockFree1, 'boolean', 'The value of `typeof isLockFree1` is "boolean"');
  // Once the values of [[Signifier]], [[IsLockFree1]], [[IsLockFree2]],
  // and [[IsLockFree8]] have been observed by any agent in the agent
  // cluster they cannot change.
  assert.sameValue(
    Atomics.isLockFree(1),
    isLockFree1,
    'Atomics.isLockFree(1) returns the value of `isLockFree1` (Atomics.isLockFree(1))'
  );
};
{
  isLockFree2 = Atomics.isLockFree(2);
  //
  // If n equals 2, return AR.[[IsLockFree2]].
  //
  assert.sameValue(typeof isLockFree2, 'boolean', 'The value of `typeof isLockFree2` is "boolean"');
  // Once the values of [[Signifier]], [[IsLockFree1]], [[IsLockFree2]],
  // and [[IsLockFree8]] have been observed by any agent in the agent
  // cluster they cannot change.
  assert.sameValue(
    Atomics.isLockFree(2),
    isLockFree2,
    'Atomics.isLockFree(2) returns the value of `isLockFree2` (Atomics.isLockFree(2))'
  );
};
{
  let isLockFree4 = Atomics.isLockFree(4);
  //
  // If n equals 4, return true.
  //
  assert.sameValue(typeof isLockFree4, 'boolean', 'The value of `typeof isLockFree4` is "boolean"');
  assert.sameValue(isLockFree4, true, 'The value of `isLockFree4` is true');
};

{
  isLockFree8 = Atomics.isLockFree(8);
  //
  // If n equals 8, return AR.[[IsLockFree8]].
  //
  assert.sameValue(typeof isLockFree8, 'boolean', 'The value of `typeof isLockFree8` is "boolean"');
  // Once the values of [[Signifier]], [[IsLockFree1]], [[IsLockFree2]],
  // and [[IsLockFree8]] have been observed by any agent in the agent
  // cluster they cannot change.
  assert.sameValue(
    Atomics.isLockFree(8),
    isLockFree8,
    'Atomics.isLockFree(8) returns the value of `isLockFree8` (Atomics.isLockFree(8))'
  );
};

{
  for (let i = 0; i < 12; i++) {
    if (![1, 2, 4, 8].includes(i)) {
      assert.sameValue(Atomics.isLockFree(i), false, 'Atomics.isLockFree(i) returns false');
    }
  }
};

assert.sameValue(
  Atomics.isLockFree(1),
  isLockFree1,
  'Atomics.isLockFree(1) returns the value of `isLockFree1` (Atomics.isLockFree(1))'
);
assert.sameValue(
  Atomics.isLockFree(2),
  isLockFree2,
  'Atomics.isLockFree(2) returns the value of `isLockFree2` (Atomics.isLockFree(2))'
);
assert.sameValue(
  Atomics.isLockFree(8),
  isLockFree8,
  'Atomics.isLockFree(8) returns the value of `isLockFree8` (Atomics.isLockFree(8))'
);

// Duplicates behavior created by loop from above
assert.sameValue(Atomics.isLockFree(3), false, 'Atomics.isLockFree(3) returns false');
assert.sameValue(Atomics.isLockFree(4), true, 'Atomics.isLockFree(4) returns true');
assert.sameValue(Atomics.isLockFree(5), false, 'Atomics.isLockFree(5) returns false');
assert.sameValue(Atomics.isLockFree(6), false, 'Atomics.isLockFree(6) returns false');
assert.sameValue(Atomics.isLockFree(7), false, 'Atomics.isLockFree(7) returns false');
assert.sameValue(Atomics.isLockFree(9), false, 'Atomics.isLockFree(9) returns false');
assert.sameValue(Atomics.isLockFree(10), false, 'Atomics.isLockFree(10) returns false');
assert.sameValue(Atomics.isLockFree(11), false, 'Atomics.isLockFree(11) returns false');
assert.sameValue(Atomics.isLockFree(12), false, 'Atomics.isLockFree(12) returns false');
