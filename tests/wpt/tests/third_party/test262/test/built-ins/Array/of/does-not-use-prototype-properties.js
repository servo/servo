// Copyright (c) 2015 the V8 project authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
/*---
esid: sec-array.of
es6id: 22.1.2.3
description: Array.of does not use prototype properties for arguments.
info: |
  It defines elements rather than assigning to them.
---*/

Object.defineProperty(Array.prototype, "0", {
  set: function(v) {
    throw new Test262Error('Should define own properties');
  }
});

var arr = Array.of(true);
assert.sameValue(arr[0], true, 'The value of arr[0] is expected to be true');

function Custom() {}

Object.defineProperty(Custom.prototype, "0", {
  set: function(v) {
    throw new Test262Error('Should define own properties');
  }
});

var custom = Array.of.call(Custom, true);
assert.sameValue(custom[0], true, 'The value of custom[0] is expected to be true');
