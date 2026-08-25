// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-construct-argumentslist-newtarget
description: >
    Throws a TypeError if trap result is not an Object: Boolean
info: |
    [[Construct]] ( argumentsList, newTarget)

    11. If Type(newObj) is not Object, throw a TypeError exception.
features: [Proxy]
---*/

var P = new Proxy(function() {
  throw new Test262Error('target should not be called');
}, {
  construct: function() {
    return true;
  }
});

assert.throws(TypeError, function() {
  new P();
});
