// Copyright 2011-2012 Norbert Lindenberg. All rights reserved.
// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.1.2
description: Tests that Intl.DateTimeFormat can be subclassed.
author: Norbert Lindenberg
includes: [compareArray.js]
---*/

// get a date-time format and have it format an array of dates for comparison with the subclass
var locales = ["tlh", "id", "en"];
var a = [new Date(0), Date.now(), new Date(Date.parse("1989-11-09T17:57:00Z"))];
var referenceDateTimeFormat = new Intl.DateTimeFormat(locales);
var referenceFormatted = a.map(referenceDateTimeFormat.format);

class MyDateTimeFormat extends Intl.DateTimeFormat {
  constructor(locales, options) {
    super(locales, options);
    // could initialize MyDateTimeFormat properties
  }
  // could add methods to MyDateTimeFormat.prototype
}

var format = new MyDateTimeFormat(locales);
var actual = a.map(format.format);
assert.compareArray(actual, referenceFormatted);
