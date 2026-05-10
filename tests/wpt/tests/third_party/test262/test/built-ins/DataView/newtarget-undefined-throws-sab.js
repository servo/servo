// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview-buffer-byteoffset-bytelength
description: >
  Throws a TypeError if NewTarget is undefined.
info: |
  24.2.2.1 DataView (buffer, byteOffset, byteLength )

  1. If NewTarget is undefined, throw a TypeError exception.
  ...
features: [SharedArrayBuffer]
---*/

var obj = {
  valueOf: function() {
    throw new Test262Error("NewTarget should be verified before byteOffset");
  }
};

var buffer = new SharedArrayBuffer(1);

assert.throws(TypeError, function() {
  DataView(buffer, obj);
});
