// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DisplayNames
description: >
  Return abrupt completion from an invalid style option
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
  style: 'small'
};

assert.throws(RangeError, () => {
  new Intl.DisplayNames('en', options);
}, 'small');

options.style = 'very long';

assert.throws(RangeError, () => {
  new Intl.DisplayNames('en', options);
}, 'very long');

options.style = 'full';

assert.throws(RangeError, () => {
  new Intl.DisplayNames('en', options);
}, 'full');

options.style = null;

assert.throws(RangeError, () => {
  new Intl.DisplayNames('en', options);
}, 'null');

options.style = '';

assert.throws(RangeError, () => {
  new Intl.DisplayNames('en', options);
}, 'the empty string');

options.style = ['narrow', 'short', 'long'];

assert.throws(RangeError, () => {
  new Intl.DisplayNames('en', options);
}, 'an array with the valid options is not necessarily valid');
