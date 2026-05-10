// Copyright (C) 2018 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Throws when an entry object is a primitive string.
esid: sec-object.fromentries
features: [Object.fromEntries]
---*/

assert.sameValue(typeof Object.fromEntries, 'function');
assert.throws(TypeError, function() {
  Object.fromEntries(['ab']);
});
