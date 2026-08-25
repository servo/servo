// Copyright (C) 2022 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Errors thrown by `flags` accessor are forwarded to the runtime
esid: sec-regexp.prototype-@@replace
info: |
    1. Let _rx_ be the *this* value.
    2. If Type(_rx_) is not Object, throw a *TypeError* exception.
    3. Let _S_ be ? ToString(_string_).
    4. Let _lengthS_ be the number of code unit elements in _S_.
    5. Let _functionalReplace_ be IsCallable(_replaceValue_).
    6. If _functionalReplace_ is *false*, then
      a. Set _replaceValue_ to ? ToString(_replaceValue_).
        i. Let _flags_ be ? ToString(? Get(_rx_, *"flags"*)).
features: [Symbol.replace]
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
  RegExp.prototype[Symbol.replace].call(obj);
});
