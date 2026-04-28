// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.freeze
description: >
  O.[[PreventExtensions]]() returns abrupt completion.
info: |
  Object.freeze ( O )

  ...
  2. Let status be ? SetIntegrityLevel(O, frozen).

  SetIntegrityLevel ( O, level )

  ...
  3. Let status be ? O.[[PreventExtensions]]().
features: [Proxy]
---*/

var p = new Proxy({}, {
  preventExtensions: function() {
    throw new Test262Error();
  },
});

assert.throws(Test262Error, function() {
  Object.freeze(p);
});
