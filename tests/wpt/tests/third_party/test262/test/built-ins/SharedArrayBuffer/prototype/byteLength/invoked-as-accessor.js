// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Requires this value to have a [[ArrayBufferData]] internal slot
features: [SharedArrayBuffer]
---*/

assert.throws(TypeError, function() {
  SharedArrayBuffer.prototype.byteLength;
});
