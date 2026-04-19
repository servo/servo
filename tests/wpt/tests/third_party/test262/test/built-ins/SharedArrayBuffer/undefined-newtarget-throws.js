// Copyright (C) 2015 André Bargull. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-sharedarraybuffer-length
description: >
  Throws a TypeError if SharedArrayBuffer is called as a function.
info: |
  SharedArrayBuffer( length )

  SharedArrayBuffer called with argument length performs the following steps:

  1. If NewTarget is undefined, throw a TypeError exception.
  ...
features: [SharedArrayBuffer]
---*/

assert.throws(TypeError, function() {
  SharedArrayBuffer();
});

assert.throws(TypeError, function() {
  SharedArrayBuffer(10);
});
