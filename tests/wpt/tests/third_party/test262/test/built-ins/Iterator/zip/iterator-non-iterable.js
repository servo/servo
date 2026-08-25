// Copyright (C) 2025 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zip
description: >
  Throws a TypeError when the "iterables" argument is not iterable.
features: [joint-iteration]
---*/

var invalidIterables = [
  Object.create(null),
  Object.create(null, {
    next: { value: function(){} },
    return: { value: function(){} },
  }),
];

// Throws a TypeError for invalid iterables values.
for (var iterables of invalidIterables) {
  assert.throws(TypeError, function() {
    Iterator.zip(iterables);
  });
}
