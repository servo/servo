// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zipkeyed
description: >
  The "mode" option must be undefined or a valid string mode.
info: |
  Iterator.zipKeyed ( iterables [ , options ] )
    ...
    3. Let mode be ? Get(options, "mode").
    4. If mode is undefined, set mode to "shortest".
    5. If mode is not one of "shortest", "longest", or "strict", throw a TypeError exception.
    ...
features: [joint-iteration]
---*/

var validModes = [
  undefined,
  "shortest",
  "longest",
  "strict",
];

var invalidModes = [
  null,
  false,
  "",
  "short",
  "long",
  "loose",
  Symbol(),
  123,
  123n,
  {},
];

// Absent "mode" option.
Iterator.zipKeyed({}, {});

// All valid mode values are accepted.
for (var mode of validModes) {
  Iterator.zipKeyed({}, {mode});
}

// Throws a TypeError for invalid mode options.
for (var mode of invalidModes) {
  assert.throws(TypeError, function() {
    Iterator.zipKeyed({}, {mode});
  });
}

// "padding" option is not retrieved when "mode" option is invalid.
for (var mode of invalidModes) {
  var options = {
    mode,
    get padding() {
      throw new Test262Error();
    }
  };
  assert.throws(TypeError, function() {
    Iterator.zipKeyed({}, options);
  });
}

// String wrappers are not accepted.
for (var mode of validModes) {
  var options = {mode: new String(mode)};
  assert.throws(TypeError, function() {
    Iterator.zipKeyed({}, options);
  });
}

// Does not call any of `toString`, `valueOf`, `Symbol.toPrimitive`.
var badMode = {
  toString() {
    throw new Test262Error();
  },
  valueOf() {
    throw new Test262Error();
  },
  [Symbol.toPrimitive]() {
    throw new Test262Error();
  },
};
assert.throws(TypeError, function() {
  Iterator.zipKeyed({}, {mode: badMode});
});
