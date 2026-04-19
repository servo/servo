// Copyright 2013 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
es5id: 15.5.4.9_3
description: >
    Tests that String.prototype.localeCompare treats a missing  "that"
    argument, undefined, and "undefined" as equivalent.
author: Norbert Lindenberg
---*/

var thisValues = ["a", "t", "u", "undefined", "UNDEFINED", "nicht definiert", "xyz", "未定义"];

var i;
for (i = 0; i < thisValues.length; i++) {
  var thisValue = thisValues[i];
  if (thisValue.localeCompare() !== thisValue.localeCompare(undefined)) {
    throw new Test262Error("String.prototype.localeCompare does not treat missing 'that' argument as undefined.");
  }
  if (thisValue.localeCompare(undefined) !== thisValue.localeCompare("undefined")) {
    throw new Test262Error("String.prototype.localeCompare does not treat undefined 'that' argument as \"undefined\".");
  }
}
