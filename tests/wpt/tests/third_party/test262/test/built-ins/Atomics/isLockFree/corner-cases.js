// Copyright (C) 2017 Mozilla Corporation.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.islockfree
description: >
  Test isLockFree on various non-intuitive arguments
features: [Atomics]
---*/

assert.sameValue(
  Atomics.isLockFree(hide(3, Number.NaN)),
  false,
  'Atomics.isLockFree(hide(3, Number.NaN)) returns false'
);
assert.sameValue(
  Atomics.isLockFree(hide(3, -1)),
  false,
  'Atomics.isLockFree(hide(3, -1)) returns false'
);
assert.sameValue(
  Atomics.isLockFree(hide(3, 3.14)),
  false,
  'Atomics.isLockFree(hide(3, 3.14)) returns false'
);
assert.sameValue(
  Atomics.isLockFree(hide(3, 0)),
  false,
  'Atomics.isLockFree(hide(3, 0)) returns false'
);

assert.sameValue(
  Atomics.isLockFree('1'),
  Atomics.isLockFree(1),
  'Atomics.isLockFree(\'1\') returns Atomics.isLockFree(1)'
);
assert.sameValue(
  Atomics.isLockFree('3'),
  Atomics.isLockFree(3),
  'Atomics.isLockFree(\'3\') returns Atomics.isLockFree(3)'
);

assert.sameValue(
  Atomics.isLockFree(true),
  Atomics.isLockFree(1),
  'Atomics.isLockFree(true) returns Atomics.isLockFree(1)'
);

assert.sameValue(
  Atomics.isLockFree(1),
  Atomics.isLockFree({valueOf: () => 1}),
  'Atomics.isLockFree(1) returns Atomics.isLockFree({valueOf: () => 1})'
);
assert.sameValue(
  Atomics.isLockFree(3),
  Atomics.isLockFree({valueOf: () => 3}),
  'Atomics.isLockFree(3) returns Atomics.isLockFree({valueOf: () => 3})'
);
assert.sameValue(
  Atomics.isLockFree(1),
  Atomics.isLockFree({toString: () => '1'}),
  'Atomics.isLockFree(1) returns Atomics.isLockFree({toString: () => \'1\'})'
);
assert.sameValue(
  Atomics.isLockFree(3),
  Atomics.isLockFree({toString: () => '3'}),
  'Atomics.isLockFree(3) returns Atomics.isLockFree({toString: () => \'3\'})'
);

function hide(k, x) {
  if (k) {
    return hide(k - 3, x) + x;
  }
  return 0;
}
