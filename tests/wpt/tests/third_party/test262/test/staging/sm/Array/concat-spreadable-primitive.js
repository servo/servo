/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/

// Primitive values should never be tried to spread
let primitives = [
    10,
    false,
    Symbol()
    // Can't change String.prototype.length
];

for (let value of primitives) {
    let prototype = Object.getPrototypeOf(value);
    prototype[Symbol.isConcatSpreadable] = true;

    Object.defineProperty(prototype, "length", {
        configurable: true,
        get() {
            // Should never invoke length getter
            assert.sameValue(true, false);
        },
    });

    let x = [1, 2].concat(value);
    assert.compareArray(x, [1, 2, value]);

    delete prototype[Symbol.isConcatSpreadable];
    delete prototype.length;

    prototype.length;
}
