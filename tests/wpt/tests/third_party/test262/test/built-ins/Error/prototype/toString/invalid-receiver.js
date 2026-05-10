// Copyright (C) 2019 Ecma International. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-error.prototype.tostring
description: >
  Error.prototype.toString throws if its receiver is not an object.
info: |
  Error.prototype.toString ( )
  1. Let O be this value.
  2. If Type(O) is not Object, throw a TypeError exception.
---*/

[undefined, null, 1, true, 'string', Symbol()].forEach((v) => {
  assert.throws(TypeError, () => {
    Error.prototype.toString.call(v);
  }, `Error.prototype.toString.call(${String(v)})`);
});
