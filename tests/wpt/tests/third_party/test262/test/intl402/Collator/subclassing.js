// Copyright 2011-2012 Norbert Lindenberg. All rights reserved.
// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.1.2_a
description: Tests that Intl.Collator can be subclassed.
author: Norbert Lindenberg
includes: [compareArray.js]
---*/

// get a collator and have it sort an array for comparison with the subclass
var locales = ["tlh", "id", "en"];
var a = ["A", "C", "E", "B", "D", "F"];
var referenceCollator = new Intl.Collator(locales);
var referenceSorted = a.slice().sort(referenceCollator.compare);

class MyCollator extends Intl.Collator {
  constructor(locales, options) {
    super(locales, options);
    // could initialize MyCollator properties
  }
  // could add methods to MyCollator.prototype
}

var collator = new MyCollator(locales);
a.sort(collator.compare);
assert.compareArray(a, referenceSorted);
