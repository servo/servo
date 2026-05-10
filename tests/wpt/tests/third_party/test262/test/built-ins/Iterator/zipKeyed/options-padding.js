// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zipkeyed
description: >
  The "padding" option must be undefined or an object.
info: |
  Iterator.zipKeyed ( iterables [ , options ] )
    ...
    6. Let paddingOption be undefined.
    7. If mode is "longest", then
      a. Set paddingOption to ? Get(options, "padding").
      b. If paddingOption is not undefined and paddingOption is not an Object, throw a TypeError exception.
    ...
features: [joint-iteration]
---*/

var validPadding = [
  undefined,
  {},
];

var invalidPadding = [
  null,
  false,
  "",
  Symbol(),
  123,
  123n,
];

// Absent "padding" option.
Iterator.zipKeyed({}, {mode: "longest"});

// All valid padding values are accepted.
for (var padding of validPadding) {
  Iterator.zipKeyed({}, {mode: "longest", padding});
}

// Throws a TypeError for invalid padding options.
for (var padding of invalidPadding) {
  assert.throws(TypeError, function() {
    Iterator.zipKeyed({}, {mode: "longest", padding});
  });
}

// Invalid padding options are okay when mode is not "longest" because the padding option is not read.
for (var padding of invalidPadding) {
  Iterator.zipKeyed({}, {padding});
  Iterator.zipKeyed({}, {mode: undefined, padding});
  Iterator.zipKeyed({}, {mode: "shortest", padding});
  Iterator.zipKeyed({}, {mode: "strict", padding});
}
