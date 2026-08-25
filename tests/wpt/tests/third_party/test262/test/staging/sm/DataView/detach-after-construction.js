// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [detachArrayBuffer.js]
description: |
  pending
esid: pending
---*/

var buf = new ArrayBuffer([1,2]);
var bufView = new DataView(buf);

$DETACHBUFFER(buf);

assert.throws(TypeError, () => bufView.getInt8(0));
