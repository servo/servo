// Copyright (C) 2019 Ecma International. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.seal
description: >
  Object.seal throws if SetIntegrityLevel(O, sealed) returns false.
info: |
  Object.seal ( O )
  ...
  2. Let status be ? SetIntegrityLevel(O, sealed).
  3. If status is false, throw a TypeError exception.

  SetIntegrityLevel ( O, level )
  ...
  3. Let status be ? O.[[PreventExtensions]]().
  4. If status is false, return false.
---*/

const p = new Proxy({}, {
  preventExtensions() {
    return false;
  },
});

assert.throws(TypeError, () => {
  Object.seal(p);
});
