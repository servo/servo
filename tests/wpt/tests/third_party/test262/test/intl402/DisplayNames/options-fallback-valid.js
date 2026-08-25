// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DisplayNames
description: >
  Valid options for fallback
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
  ...
  15. Let fallback be ? GetOption(options, "fallback", "string", « "code", "none" », "code").
  ...

  GetOption ( options, property, type, values, fallback )

  1. Let value be ? Get(options, property).
  ...
features: [Intl.DisplayNames]
locale: [en]
---*/

// results for option values verified in the tests for resolvedOptions

const fallbacks = [
  undefined,
  'code',
  'none'
];

fallbacks.forEach(fallback => {
  const obj = new Intl.DisplayNames('en', { fallback, type: 'language'});

  assert(obj instanceof Intl.DisplayNames, `instanceof check - ${fallback}`);
  assert.sameValue(Object.getPrototypeOf(obj), Intl.DisplayNames.prototype, `proto check - ${fallback}`);
});
