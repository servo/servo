// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DisplayNames
description: >
  Valid options for localeMatcher
info: |
  Intl.DisplayNames ( locales , options )

  1. If NewTarget is undefined, throw a TypeError exception.
  2. Let displayNames be ? OrdinaryCreateFromConstructor(NewTarget, "%DisplayNamesPrototype%",
    « [[InitializedDisplayNames]], [[Locale]], [[Style]], [[Type]], [[Fallback]], [[Fields]] »).
  ...
  4. Let options be ? ToObject(options).
  ...
  12. Let type be ? GetOption(options, "type", "string", « "language", "region", "script", "currency" », undefined).
  13. If type is undefined, throw a TypeError exception.
  ...

  GetOption ( options, property, type, values, fallback )

  1. Let value be ? Get(options, property).
  ...
features: [Intl.DisplayNames]
locale: [en]
---*/

// results for option values verified in the tests for resolvedOptions

const types = [
  'language',
  'region',
  'script',
  'currency'
];

types.forEach(type => {
  const obj = new Intl.DisplayNames('en', { type });

  assert(obj instanceof Intl.DisplayNames, `instanceof check - ${type}`);
  assert.sameValue(Object.getPrototypeOf(obj), Intl.DisplayNames.prototype, `proto check - ${type}`);
});
