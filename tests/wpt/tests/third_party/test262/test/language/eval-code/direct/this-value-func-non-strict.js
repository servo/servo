// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Direct eval code has the same `this` binding as the calling context
    (non-strict function scope)
esid: sec-performeval
flags: [noStrict]
---*/

var thisValue;

(function() {
  thisValue = eval('this;');
}());

assert.sameValue(thisValue, this);
