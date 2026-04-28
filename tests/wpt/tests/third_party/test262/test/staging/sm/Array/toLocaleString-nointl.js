// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
if (typeof Intl !== "object") {
    const localeSep = [,,].toLocaleString();

    const obj = {
        toLocaleString() {
            assert.sameValue(arguments.length, 0);
            return "pass";
        }
    };

    // Ensure no arguments are passed to the array elements.
    // - Single element case.
    assert.sameValue([obj].toLocaleString(), "pass");
    // - More than one element.
    assert.sameValue([obj, obj].toLocaleString(), "pass" + localeSep + "pass");

    // Ensure no arguments are passed to the array elements even if supplied.
    const locales = {}, options = {};
    // - Single element case.
    assert.sameValue([obj].toLocaleString(locales, options), "pass");
    // - More than one element.
    assert.sameValue([obj, obj].toLocaleString(locales, options), "pass" + localeSep + "pass");
}

