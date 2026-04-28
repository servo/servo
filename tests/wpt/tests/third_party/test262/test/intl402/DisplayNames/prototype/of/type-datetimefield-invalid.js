// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-Intl.DisplayNames.prototype.of
description: Throws a RangeError for invalid `dateTimeField` codes
features: [Intl.DisplayNames-v2]
---*/

var displayNames = new Intl.DisplayNames(undefined, {type: 'dateTimeField'});

assert.throws(RangeError, function() {
  displayNames.of('');
}, 'empty string');

assert.throws(RangeError, function() {
  displayNames.of('timezoneName');
}, 'timezoneName');

assert.throws(RangeError, function() {
  displayNames.of('timezonename');
}, 'timezonename');

assert.throws(RangeError, function() {
  displayNames.of('millisecond');
}, 'millisecond');

assert.throws(RangeError, function() {
  displayNames.of('seconds');
}, 'seconds');

assert.throws(RangeError, function() {
  displayNames.of(' year');
}, 'year (with leading space)');

assert.throws(RangeError, function() {
  displayNames.of('year ');
}, 'year (with trailing space)');
