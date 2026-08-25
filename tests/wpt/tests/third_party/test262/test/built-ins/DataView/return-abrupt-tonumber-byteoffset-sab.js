// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview-buffer-byteoffset-bytelength
description: >
  Return abrupt from ToNumber(byteOffset)
info: |
  24.2.2.1 DataView (buffer, byteOffset, byteLength )

  ...
  4. Let numberOffset be ? ToNumber(byteOffset).
  ...
features: [SharedArrayBuffer]
---*/

var obj = {
  valueOf: function() {
    throw new Test262Error();
  }
};

var ab = new SharedArrayBuffer(0);

assert.throws(Test262Error, function() {
  new DataView(ab, obj);
});
