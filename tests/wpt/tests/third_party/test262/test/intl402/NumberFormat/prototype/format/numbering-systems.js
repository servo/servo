// Copyright 2012 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: table-numbering-system-digits
description: >
    Tests that Intl.NumberFormat.prototype.format supports all
    numbering systems with simple digit mappings.
author: Roozbeh Pournader
includes: [testIntl.js]
---*/

for (let [numberingSystem, digits] of Object.entries(numberingSystemDigits)) {
    let digitList = [...digits];
    assert.sameValue(digitList.length, 10);

    let nf = new Intl.NumberFormat(undefined, {numberingSystem});

    for (let i = 0; i <= 9; ++i) {
        assert.sameValue(nf.format(i), digitList[i],
                         `numberingSystem: ${numberingSystem}, digit: ${i}`);
    }
}
