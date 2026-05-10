// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-construct-argumentslist-newtarget
description: >
    Return the result from the trap method.
info: |
    [[Construct]] ( argumentsList, newTarget)

    12. Return newObj
features: [Proxy]
---*/

var P = new Proxy(function() {
  throw new Test262Error('target should not be called');
}, {
  construct: function(t, c, args) {
    return {
      sum: 42
    };
  }
});

assert.sameValue((new P(1, 2)).sum, 42);
