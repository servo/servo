// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zipkeyed
description: >
  Throws a TypeError when the "iterables" argument is not an object.
info: |
  Iterator.zipKeyed ( iterables [ , options ] )
    1. If iterables is not an Object, throw a TypeError exception.
    ...
features: [joint-iteration]
---*/

var invalidIterables = [
  undefined,
  null,
  true,
  "",
  Symbol(),
  0,
  0n,
];

// Throws when the "iterables" argument is absent.
assert.throws(TypeError, function() {
  Iterator.zipKeyed();
});

// Throws a TypeError for invalid iterables values.
for (var iterables of invalidIterables) {
  assert.throws(TypeError, function() {
    Iterator.zipKeyed(iterables);
  });
}

// Options argument not read when iterables is not an object.
var badOptions = {
  get mode() {
    throw new Test262Error();
  },
  get padding() {
    throw new Test262Error();
  }
};
for (var iterables of invalidIterables) {
  assert.throws(TypeError, function() {
    Iterator.zipKeyed(iterables, badOptions);
  });
}
