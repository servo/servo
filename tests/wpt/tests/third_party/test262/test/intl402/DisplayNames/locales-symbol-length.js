// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DisplayNames
description: >
  CanonicalizeLocaleList tries to fetch length from Object.
info: |
  Intl.DisplayNames ( locales , options )

  1. If NewTarget is undefined, throw a TypeError exception.
  2. Let displayNames be ? OrdinaryCreateFromConstructor(NewTarget, "%DisplayNamesPrototype%",
    « [[InitializedDisplayNames]], [[Locale]], [[Style]], [[Type]], [[Fallback]], [[Fields]] »).
  3. Let requestedLocales be ? CanonicalizeLocaleList(locales).
  ...
  12. Let type be ? GetOption(options, "type", "string", « "language", "region", "script", "currency" », undefined).
  13. If type is undefined, throw a TypeError exception.
  ...

  CanonicalizeLocaleList ( locales )

  1. If locales is undefined, then
    a. Return a new empty List.
  2. Let seen be a new empty List.
  3. If Type(locales) is String, then
    a. Let O be CreateArrayFromList(« locales »).
  4. Else,
    a. Let O be ? ToObject(locales).
  5. Let len be ? ToLength(? Get(O, "length")).
features: [Intl.DisplayNames, Symbol]
locale: [en]
includes: [compareArray.js]
---*/

let calls = [];
let symbol = Symbol();

Symbol.prototype.length = 1;

Object.defineProperty(Symbol.prototype, 'length', {
  get() {
    assert.notSameValue(this, symbol, 'this is an object from given symbol');
    assert.sameValue(this.valueOf(), symbol, 'internal value is the symbol');
    assert(this instanceof Symbol);
    calls.push('length');
    return 1;
  }
});

Object.defineProperty(Symbol.prototype, '0', {
  get() {
    assert.notSameValue(this, symbol, 'this is an object from given symbol');
    assert.sameValue(this.valueOf(), symbol, 'internal value is the symbol');
    assert(this instanceof Symbol);
    calls.push('0');
    return 'en';
  }
});

new Intl.DisplayNames(symbol, {type: 'language'});

assert.compareArray(calls, ['length', '0']);
