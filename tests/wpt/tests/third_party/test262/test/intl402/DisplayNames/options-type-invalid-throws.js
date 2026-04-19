// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DisplayNames
description: >
  Return abrupt completion from an invalid type option
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

assert.throws(TypeError, () => {
  new Intl.DisplayNames('en', undefined);
}, 'undefined options');

assert.throws(TypeError, () => {
  new Intl.DisplayNames('en', {});
}, '{} options');

assert.throws(TypeError, () => {
  new Intl.DisplayNames('en', {type: undefined});
}, 'undefined type');

assert.throws(RangeError, () => {
  new Intl.DisplayNames('en', {type: 'lang'});
}, 'type = lang');

assert.throws(RangeError, () => {
  new Intl.DisplayNames('en', {type: 'dayPeriod'});
}, 'dayPeriod not supported yet');

assert.throws(RangeError, () => {
  new Intl.DisplayNames('en', {type: 'weekday'});
}, 'weekday not supported yet');

assert.throws(RangeError, () => {
  new Intl.DisplayNames('en', {type: null});
}, 'type = null');


assert.throws(RangeError, () => {
  new Intl.DisplayNames('en', {type: ''});
}, 'type = ""');

assert.throws(RangeError, () => {
  new Intl.DisplayNames('en', {type: ['language', 'region', 'script', 'currency']});
}, 'an array with the valid options is not necessarily valid');
