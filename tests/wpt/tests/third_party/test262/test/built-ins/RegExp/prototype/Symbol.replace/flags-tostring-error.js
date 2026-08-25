// Copyright (C) 2022 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Errors thrown by converting `flags` to string are forwarded to the runtime
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
var toStringThrows = {
  [Symbol.toPrimitive](hint) {
    if (hint === 'string') {
      throw new CustomError();
    }
    throw new Test262Error('@@toPrimitive should be called with hint "string"');
  },
  get toString() { throw new Test262Error('toString property should not be read'); },
  get valueOf() { throw new Test262Error('valueOf property should not be read'); }
};

var re = /./g;
Object.defineProperties(re, {
  flags: {
    get() { return toStringThrows; }
  },
  global: {
    get() { throw new Test262Error('global property should not be read'); }
  },
  unicode: {
    get() { throw new Test262Error('unicode property should not be read'); }
  }
});

assert.throws(CustomError, function() {
  re[Symbol.replace]('');
});
