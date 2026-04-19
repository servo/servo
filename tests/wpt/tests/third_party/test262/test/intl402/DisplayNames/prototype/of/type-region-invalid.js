// Copyright 2023 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-Intl.DisplayNames.prototype.of
description: Throws a RangeError for invalid `region` codes
info: |
  12.5.1 CanonicalCodeForDisplayNames ( code )

  ...
  2. If type is "region", then
    a. If code cannot be matched by the unicode_region_subtag Unicode locale nonterminal, throw a RangeError exception.
    b. Return the ASCII-uppercase of code.
features: [Intl.DisplayNames]
---*/

// https://unicode.org/reports/tr35/#unicode_region_subtag
// unicode_region_subtag = (alpha{2} | digit{3}) ;

var displayNames = new Intl.DisplayNames(undefined, {type: 'region'});

assert.throws(RangeError, function() {
  displayNames.of('00');
}, 'insufficient length, numeric');

assert.throws(RangeError, function() {
  displayNames.of('a');
}, 'insufficient length, alpha');

assert.throws(RangeError, function() {
  displayNames.of('aaa');
}, 'excessive length, alpha');

assert.throws(RangeError, function() {
  displayNames.of('1111');
}, 'excessive length, numeric');

assert.throws(RangeError, function() {
  displayNames.of('');
}, 'empty string');

assert.throws(RangeError, function() {
  displayNames.of('a01');
}, 'mixed alphanumeric (alpha first, length 3)');

assert.throws(RangeError, function() {
  displayNames.of('a1');
}, 'mixed alphanumeric (alpha first, length 2)');

assert.throws(RangeError, function() {
  displayNames.of('1a');
}, 'mixed alphanumeric (numeric first, length 2)');

assert.throws(RangeError, function() {
  displayNames.of('1a1');
}, 'mixed alphanumeric (numeric first, length 3)');

assert.throws(RangeError, function() {
  displayNames.of('-111');
}, 'leading separator (dash)');

assert.throws(RangeError, function() {
  displayNames.of('_111');
}, 'leading separator (underscore)');

assert.throws(RangeError, function() {
  displayNames.of('111-');
}, 'trailing separator (dash)');

assert.throws(RangeError, function() {
  displayNames.of('111-');
}, 'trailing separator (underscore)');

assert.throws(RangeError, function() {
  displayNames.of(' aa');
}, 'leading space');

assert.throws(RangeError, function() {
  displayNames.of('aa ');
}, 'trailing space');

assert.throws(RangeError, function() {
  displayNames.of('a c');
}, 'interstitial space');
