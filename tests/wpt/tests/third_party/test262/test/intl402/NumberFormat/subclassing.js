// Copyright 2011-2012 Norbert Lindenberg. All rights reserved.
// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.1.2
description: Tests that Intl.NumberFormat can be subclassed.
author: Norbert Lindenberg
includes: [compareArray.js]
---*/

// get a number format and have it format an array of numbers for comparison with the subclass
var locales = ["tlh", "id", "en"];
var a = [0, 1, -1, -123456.789, -Infinity, NaN];
var referenceNumberFormat = new Intl.NumberFormat(locales);
var referenceFormatted = a.map(referenceNumberFormat.format);

class MyNumberFormat extends Intl.NumberFormat {
  constructor(locales, options) {
    super(locales, options);
    // could initialize MyNumberFormat properties
  }
  // could add methods to MyNumberFormat.prototype
}

var format = new MyNumberFormat(locales);
var actual = a.map(format.format);
assert.compareArray(actual, referenceFormatted);
