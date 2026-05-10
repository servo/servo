// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-sharedarraybuffer-length
description: >
  Throws a TypeError if length is a symbol
info: |
  SharedArrayBuffer( length )

  1. If NewTarget is undefined, throw a TypeError exception.
  2. Let byteLength be ? ToIndex(length).
  ...
features: [SharedArrayBuffer, Symbol]
---*/

var s = Symbol();

assert.throws(TypeError, function() {
  new SharedArrayBuffer(s);
}, "`length` parameter is a Symbol");
