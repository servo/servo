// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-Intl.DisplayNames.prototype.of
description: Throws a RangeError for invalid `calendar` codes
features: [Intl.DisplayNames-v2]
---*/

var displayNames = new Intl.DisplayNames(undefined, {type: 'calendar'});

assert.throws(RangeError, function() {
  displayNames.of('00');
}, 'insufficient length');

assert.throws(RangeError, function() {
  displayNames.of('000000000');
}, 'excessive length');

assert.throws(RangeError, function() {
  displayNames.of('-00000000');
}, 'leading separator (dash)');

assert.throws(RangeError, function() {
  displayNames.of('_00000000');
}, 'leading separator (underscore)');

assert.throws(RangeError, function() {
  displayNames.of('00000000-');
}, 'trailing separator (dash)');

assert.throws(RangeError, function() {
  displayNames.of('00000000_');
}, 'trailing separator (underscore)');

assert.throws(RangeError, function() {
  displayNames.of(' abcdef');
}, 'leading space');

assert.throws(RangeError, function() {
  displayNames.of('abcdef ');
}, 'trailing space');

assert.throws(RangeError, function() {
  displayNames.of('abc def');
}, 'interstitial space');

assert.throws(RangeError, function() {
  displayNames.of('123_abc');
}, '2 segments, minimum length, underscore');

assert.throws(RangeError, function() {
  displayNames.of('12345678_abcdefgh');
}, '2 segments, maximum length, underscore');

assert.throws(RangeError, function() {
  displayNames.of('123_abc_ABC');
}, '3 segments, minimum length, underscore');

assert.throws(RangeError, function() {
  displayNames.of('12345678_abcdefgh_ABCDEFGH');
}, '3 segments, maximum length, underscore');
