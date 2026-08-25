// Copyright 2013 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
es5id: 13.1.1_3_2
description: >
    Tests that String.prototype.localeCompare treats a missing  "that"
    argument, undefined, and "undefined" as equivalent.
author: Norbert Lindenberg
---*/

var thisValues = ["a", "t", "u", "undefined", "UNDEFINED", "nicht definiert", "xyz", "未定义"];

var i;
for (i = 0; i < thisValues.length; i++) {
    var thisValue = thisValues[i];
    assert.sameValue(thisValue.localeCompare(), thisValue.localeCompare(undefined), "String.prototype.localeCompare does not treat missing 'that' argument as undefined.");
    assert.sameValue(thisValue.localeCompare(undefined), thisValue.localeCompare("undefined"), "String.prototype.localeCompare does not treat undefined 'that' argument as \"undefined\".");
}
