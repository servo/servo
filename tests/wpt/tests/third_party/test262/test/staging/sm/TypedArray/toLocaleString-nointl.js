// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js]
description: |
  pending
esid: pending
---*/
if (typeof Intl !== "object") {
    const localeSep = [,,].toLocaleString();

    const originalNumberToLocaleString = Number.prototype.toLocaleString;

    // Ensure no arguments are passed to the array elements.
    for (let constructor of anyTypedArrayConstructors) {
        Number.prototype.toLocaleString = function() {
            assert.sameValue(arguments.length, 0);
            return "pass";
        };

        // Single element case.
        assert.sameValue(new constructor(1).toLocaleString(), "pass");

        // More than one element.
        assert.sameValue(new constructor(2).toLocaleString(), "pass" + localeSep + "pass");
    }
    Number.prototype.toLocaleString = originalNumberToLocaleString;

    // Ensure no arguments are passed to the array elements even if supplied.
    for (let constructor of anyTypedArrayConstructors) {
        Number.prototype.toLocaleString = function() {
            assert.sameValue(arguments.length, 0);
            return "pass";
        };
        let locales = {};
        let options = {};

        // Single element case.
        assert.sameValue(new constructor(1).toLocaleString(locales, options), "pass");

        // More than one element.
        assert.sameValue(new constructor(2).toLocaleString(locales, options), "pass" + localeSep + "pass");
    }
    Number.prototype.toLocaleString = originalNumberToLocaleString;
}

