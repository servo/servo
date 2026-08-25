// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DisplayNames
description: >
  Throws TypeError if locales is not undefined, a string, or an array-like object.
info: |
  Intl.DisplayNames ([ locales [ , options ]])

  1. If NewTarget is undefined, throw a TypeError exception.
  2. Let displayNames be ? OrdinaryCreateFromConstructor(NewTarget, "%DisplayNamesPrototype%",
    « [[InitializedDisplayNames]], [[Locale]], [[Style]], [[Type]], [[Fallback]], [[Fields]] »).
  3. Let requestedLocales be ? CanonicalizeLocaleList(locales).
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
features: [Intl.DisplayNames]
---*/

assert.throws(TypeError, () => {
  new Intl.DisplayNames(null);
}, 'null');
