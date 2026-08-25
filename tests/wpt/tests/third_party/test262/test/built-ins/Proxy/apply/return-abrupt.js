// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-call-thisargument-argumentslist
description: >
    Return is an abrupt completion
features: [Proxy]
---*/

var p = new Proxy(function() {
  throw 'not the Test262Error you are looking for';
}, {
  apply: function(t, c, args) {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  p.call();
});
