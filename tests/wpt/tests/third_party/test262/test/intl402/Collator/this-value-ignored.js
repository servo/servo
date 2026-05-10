// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.1.1_1
description: Tests that the this-value is ignored in Collator.
author: Norbert Lindenberg
includes: [testIntl.js]
---*/

testWithIntlConstructors(function (Constructor) {
    var obj, newObj;

    // variant 1: use constructor in a "new" expression
    obj = new Constructor();
    newObj = Intl.Collator.call(obj);
    assert.notSameValue(obj, newObj, "Collator object created with \"new\" was not ignored as this-value.");

    // variant 2: use constructor as a function
    if (Constructor !== Intl.Collator &&
        Constructor !== Intl.NumberFormat &&
        Constructor !== Intl.DateTimeFormat)
    {
        // Newer Intl constructors are not callable as a function.
        return;
    }
    obj = Constructor();
    newObj = Intl.Collator.call(obj);
    assert.notSameValue(obj, newObj, "Collator object created with constructor as function was not ignored as this-value.");
});
