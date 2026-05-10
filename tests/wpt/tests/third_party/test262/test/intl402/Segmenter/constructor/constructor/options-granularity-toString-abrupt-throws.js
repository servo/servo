// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.segmenter
description: Return abrupt completion from GetOption granularity

info: |
    Intl.Segmenter ([ locales [ , options ]])

    13. Let granularity be ? GetOption(options, "granularity", "string", « "grapheme", "word", "sentence" », "grapheme").

    GetOption ( options, property, type, values, fallback )
    6. If type is "string", then
        a. Let value be ? ToString(value).
features: [Intl.Segmenter, Symbol]
---*/

const options = {
  granularity: {
    toString() {
      throw new Test262Error();
    }
  }
};

assert.throws(Test262Error, () => {
  new Intl.Segmenter(undefined, options);
}, 'from toString');

options.granularity = {
  toString: undefined,
  valueOf() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, () => {
  new Intl.Segmenter(undefined, options);
}, 'from valueOf');

options.granularity = {
  [Symbol.toPrimitive]() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, () => {
  new Intl.Segmenter(undefined, options);
}, 'from ToPrimitive');

options.granularity = Symbol();

assert.throws(TypeError, () => {
  new Intl.Segmenter(undefined, options);
}, 'symbol value');
