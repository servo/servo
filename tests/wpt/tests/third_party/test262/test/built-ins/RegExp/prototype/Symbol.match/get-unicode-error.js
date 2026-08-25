// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Errors thrown by `unicode` accessor are forwarded to the runtime
esid: sec-regexp.prototype-@@match
info: |
    1. Let _rx_ be the *this* value.
    2. If Type(_rx_) is not Object, throw a *TypeError* exception.
    3. Let _S_ be ? ToString(_string_).
    4. Let _flags_ be ? ToString(? Get(_rx_, *"flags"*)).

    sec-get-regexp.prototype.flags get RegExp.prototype.flags
    14. Let _unicode_ be ToBoolean(? Get(_R_, *"unicode"*)).
features: [Symbol.match]
---*/

var nonGlobalRe = /./;
var globalRe = /./g;
var accessor = function() {
  throw new Test262Error();
};
Object.defineProperty(nonGlobalRe, 'unicode', {
  get: accessor
});
Object.defineProperty(globalRe, 'unicode', {
  get: accessor
});

assert.throws(Test262Error, function() {
  nonGlobalRe[Symbol.match]('');
});

assert.throws(Test262Error, function() {
  globalRe[Symbol.match]('');
});
