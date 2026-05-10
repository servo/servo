// Copyright (C) 2019 Leo Balter. All rights reserved.
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
  10. Let style be ? GetOption(options, "style", "string", « "narrow", "short", "long" », "long").
  ...
  12. Let type be ? GetOption(options, "type", "string", « "language", "region", "script", "currency" », undefined).
  13. If type is undefined, throw a TypeError exception.

  GetOption ( options, property, type, values, fallback )

  1. Let value be ? Get(options, property).
  ...
features: [Intl.DisplayNames, Symbol]
locale: [en]
---*/

assert.throws(Test262Error, () => {
  new Intl.DisplayNames('en', {
    type: 'language', fallback: { toString() { throw new Test262Error(); }}
  });
}, 'from toString');

assert.throws(Test262Error, () => {
  new Intl.DisplayNames('en', {
    type: 'language',
    fallback: {toString: undefined, valueOf() {throw new Test262Error(); }}
  });
}, 'from valueOf');

assert.throws(Test262Error, () => {
  new Intl.DisplayNames('en', {
    type: 'language',
    fallback: { [Symbol.toPrimitive]() { throw new Test262Error(); } }
  });
}, 'from ToPrimitive');

assert.throws(TypeError, () => {
  new Intl.DisplayNames('en', {
    type: 'language',
    fallback: Symbol()
  });
}, 'symbol value');
