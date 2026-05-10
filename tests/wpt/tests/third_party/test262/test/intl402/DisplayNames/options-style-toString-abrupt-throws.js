// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DisplayNames
description: >
  Return abrupt completion from GetOption style
info: |
  Intl.DisplayNames ([ locales [ , options ]])

  1. If NewTarget is undefined, throw a TypeError exception.
  2. Let displayNames be ? OrdinaryCreateFromConstructor(NewTarget, "%DisplayNamesPrototype%",
    « [[InitializedDisplayNames]], [[Locale]], [[Style]], [[Type]], [[Fallback]], [[Fields]] »).
  ...
  4. If options is undefined, then
    a. Let options be ObjectCreate(null).
  5. Else
    a. Let options be ? ToObject(options).
  ...
  8. Let matcher be ? GetOption(options, "localeMatcher", "string", « "lookup", "best fit" », "best fit").
  ...
  11. Let style be ? GetOption(options, "style", "string", « "narrow", "short", "long" », "long").
  ...

  GetOption ( options, property, type, values, fallback )

  1. Let value be ? Get(options, property).
  ...
features: [Intl.DisplayNames, Symbol]
locale: [en]
---*/

var options = {
  style: {
    toString() {
      throw new Test262Error();
    }
  }
};

assert.throws(Test262Error, () => {
  new Intl.DisplayNames('en', options);
}, 'from toString');

options.style = {
  toString: undefined,
  valueOf() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, () => {
  new Intl.DisplayNames('en', options);
}, 'from valueOf');

options.style = {
  [Symbol.toPrimitive]() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, () => {
  new Intl.DisplayNames('en', options);
}, 'from ToPrimitive');

options.style = Symbol();

assert.throws(TypeError, () => {
  new Intl.DisplayNames('en', options);
}, 'symbol value');
