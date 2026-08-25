// Copyright (c) 2014 Thomas Dahlstrom. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.sort
description: >
    The SortCompare abstract operation calls ToString() for  identical
    elements (step 14/15)
author: Thomas Dahlstrom (tdahlstrom@gmail.com)
---*/

var counter = 0;
var object = {
  toString: function() {
    counter++;
    return "";
  }
};

[object, object].sort();
if (counter < 2) {
  // sort calls ToString() for each element at least once
  throw new Test262Error('#1: [object, object].sort(); counter < 2. Actual: ' + (counter));
}
