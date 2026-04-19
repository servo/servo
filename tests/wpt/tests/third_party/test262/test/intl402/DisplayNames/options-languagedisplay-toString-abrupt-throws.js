// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DisplayNames
description: >
  Return abrupt completion from GetOption fallback
info: |
  Intl.DisplayNames ( locales , options )

  1. If NewTarget is undefined, throw a TypeError exception.
  2. Let displayNames be ? OrdinaryCreateFromConstructor(NewTarget, "%DisplayNamesPrototype%",
    « [[InitializedDisplayNames]], [[Locale]], [[Style]], [[Type]], [[Fallback]], [[Fields]] »).
  ...
  4. Let options be ? ToObject(options).
  ...
  8. Let matcher be ? GetOption(options, "localeMatcher", "string", « "lookup", "best fit" », "best fit").
  ...
  24. Let languageDisplay be ? GetOption(options, "languageDisplay", "string", « "dialect", "standard" », "dialect").
  ...

  GetOption ( options, property, type, values, fallback )

  1. Let value be ? Get(options, property).
  ...
features: [Intl.DisplayNames-v2, Symbol]
locale: [en]
---*/

assert.throws(Test262Error, () => {
  new Intl.DisplayNames('en', {
    type: 'language', languageDisplay: { toString() { throw new Test262Error(); }}
  });
}, 'from toString');

assert.throws(Test262Error, () => {
  new Intl.DisplayNames('en', {
    type: 'language',
    languageDisplay: {toString: undefined, valueOf() {throw new Test262Error(); }}
  });
}, 'from valueOf');

assert.throws(Test262Error, () => {
  new Intl.DisplayNames('en', {
    type: 'language',
    languageDisplay: { [Symbol.toPrimitive]() { throw new Test262Error(); } }
  });
}, 'from ToPrimitive');

assert.throws(TypeError, () => {
  new Intl.DisplayNames('en', {
    type: 'language',
    languageDisplay: Symbol()
  });
}, 'symbol value');
