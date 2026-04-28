// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DisplayNames
description: >
  Return abrupt completion from an invalid localeMatcher option
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

  GetOption ( options, property, type, values, fallback )

  1. Let value be ? Get(options, property).
  2. If value is not undefined, then
    ...
    c. If type is "string", then
      i. Let value be ? ToString(value).
    d. If values is not undefined, then
      i. If values does not contain an element equal to value, throw a RangeError exception.
  ...
features: [Intl.DisplayNames]
locale: [en]
---*/

var options = {
  localeMatcher: 'bestfit' // not "best fit"
};

assert.throws(RangeError, () => {
  new Intl.DisplayNames('en', options);
}, 'bestfit');

options.localeMatcher = 'look up';

assert.throws(RangeError, () => {
  new Intl.DisplayNames('en', options);
}, 'look up');

options.localeMatcher = null;

assert.throws(RangeError, () => {
  new Intl.DisplayNames('en', options);
}, 'null');

options.localeMatcher = '';

assert.throws(RangeError, () => {
  new Intl.DisplayNames('en', options);
}, 'the empty string');

// The world could burn
options.localeMatcher = ['lookup', 'best fit'];

assert.throws(RangeError, () => {
  new Intl.DisplayNames('en', options);
}, 'an array with the valid options is not necessarily valid');
