// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zipkeyed
description: >
  The "options" argument can either be undefined or an object.
info: |
  Iterator.zipKeyed ( iterables [ , options ] )
    ...
    2. Set options to ? GetOptionsObject(options).
    ...

  GetOptionsObject ( options )
    1. If options is undefined, then
      a. Return OrdinaryObjectCreate(null).
    2. If options is an Object, then
      a. Return options.
    3. Throw a TypeError exception.
features: [joint-iteration]
---*/

var validOptions = [
  undefined,
  {},
];

var invalidOptions = [
  null,
  true,
  "",
  Symbol(),
  0,
  0n,
];

// The "options" argument can also be absent.
Iterator.zipKeyed({});

// All valid option values are accepted.
for (var options of validOptions) {
  Iterator.zipKeyed({}, options);
}

// Throws a TypeError for invalid option values.
for (var options of invalidOptions) {
  assert.throws(TypeError, function() {
    Iterator.zipKeyed({}, options);
  });
}
