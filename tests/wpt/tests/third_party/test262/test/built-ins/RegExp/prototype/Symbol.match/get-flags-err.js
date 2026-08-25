// Copyright (C) 2022 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Errors thrown by `flags` accessor are forwarded to the runtime
esid: sec-regexp.prototype-@@match
info: |
    1. Let _rx_ be the *this* value.
    2. If Type(_rx_) is not Object, throw a *TypeError* exception.
    3. Let _S_ be ? ToString(_string_).
    4. Let _flags_ be ? ToString(? Get(_rx_, *"flags"*)).
features: [Symbol.match]
---*/

function CustomError() {}

var obj = {
  get flags() {
    throw new CustomError();
  },
  get global() {
    throw new Test262Error('global property should not be read');
  },
  get unicode() {
    throw new Test262Error('unicode property should not be read');
  }
};

assert.throws(CustomError, function() {
  RegExp.prototype[Symbol.match].call(obj);
});
