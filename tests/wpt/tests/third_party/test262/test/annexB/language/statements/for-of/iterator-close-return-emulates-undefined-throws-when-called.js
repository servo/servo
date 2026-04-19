// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iteratorclose
description: >
    If <iterator>.return is an object emulating `undefined` (e.g. `document.all`
    in browsers), it shouldn't be treated as if it were actually `undefined`.
features: [generators, IsHTMLDDA]
---*/

var IsHTMLDDA = $262.IsHTMLDDA;
var iter = {
  [Symbol.iterator]() { return this; },
  next() { return {}; },
  return: IsHTMLDDA,
};

assert.throws(TypeError, function() {
  // `IsHTMLDDA` is called here with `iter` as `this` and no arguments, and it's
  // specified to return `null` under these conditions.  Then the iteration
  // protocol throws a `TypeError` because `null` isn't an object.
  for (var x of iter)
    break;
});
