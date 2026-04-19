// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl-numberformat-constructor
description: >
    Tests that the this-value is ignored in NumberFormat, if the this-value
    isn't a NumberFormat instance.
author: Norbert Lindenberg
includes: [testIntl.js]
---*/

testWithIntlConstructors(function (Constructor) {
    if (Constructor === Intl.NumberFormat)
        return;

    var obj, newObj;

    // variant 1: use constructor in a "new" expression
    obj = new Constructor();
    newObj = Intl.NumberFormat.call(obj);
    assert.notSameValue(obj, newObj, "NumberFormat object created with \"new\" was not ignored as this-value.");

    // variant 2: use constructor as a function
    if (Constructor !== Intl.Collator &&
        Constructor !== Intl.NumberFormat &&
        Constructor !== Intl.DateTimeFormat)
    {
        // Newer Intl constructors are not callable as a function.
        return;
    }
    obj = Constructor();
    newObj = Intl.NumberFormat.call(obj);
    assert.notSameValue(obj, newObj, "NumberFormat object created with constructor as function was not ignored as this-value.");
});
