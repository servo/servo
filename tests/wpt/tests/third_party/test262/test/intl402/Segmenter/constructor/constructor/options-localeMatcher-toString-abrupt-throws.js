// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.segmenter
description: >
  Return abrupt completion from GetOption localeMatcher
info: |
  Intl.Segmenter ([ locales [ , options ]])

  1. If NewTarget is undefined, throw a TypeError exception.
  3. Let segmenter be ? OrdinaryCreateFromConstructor(NewTarget, "%Segmenter.prototype%", internalSlotsList).
  ...
  4. If options is undefined, then
    a. Let options be ObjectCreate(null).
  5. Else
    a. Let options be ? ToObject(options).
  ...
  8. Let matcher be ? GetOption(options, "localeMatcher", "string", « "lookup", "best fit" », "best fit").

  GetOption ( options, property, type, values, fallback )
  6. If type is "string", then
        a. Let value be ? ToString(value).
  ...
features: [Intl.Segmenter, Symbol]
---*/

const options = {
  localeMatcher: {
    toString() {
      throw new Test262Error();
    }
  }
};

assert.throws(Test262Error, () => {
  new Intl.Segmenter(undefined, options);
}, 'from toString');

options.localeMatcher = {
  toString: undefined,
  valueOf() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, () => {
  new Intl.Segmenter(undefined, options);
}, 'from valueOf');

options.localeMatcher = {
  [Symbol.toPrimitive]() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, () => {
  new Intl.Segmenter(undefined, options);
}, 'from ToPrimitive');

options.localeMatcher = Symbol();

assert.throws(TypeError, () => {
  new Intl.Segmenter(undefined, options);
}, 'symbol value');
