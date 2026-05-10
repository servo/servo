// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-createdatetimeformat
description: >
  Invalid values for the `timeZoneName` option of the DateTimeFormat constructor
features: [Intl.DateTimeFormat-extend-timezonename]
---*/

assert.throws(RangeError, function () {
  new Intl.DateTimeFormat('en', { timeZoneName: '' });
}, 'empty string');

assert.throws(RangeError, function () {
  new Intl.DateTimeFormat('en', { timeZoneName: 'short ' });
}, '"short "');

assert.throws(RangeError, function () {
  new Intl.DateTimeFormat('en', { timeZoneName: ' long' });
}, '" long"');

assert.throws(RangeError, function () {
  new Intl.DateTimeFormat('en', { timeZoneName: 'offset' });
}, '"offset"');

assert.throws(RangeError, function () {
  new Intl.DateTimeFormat('en', { timeZoneName: 'generic' });
}, '"generic"');
