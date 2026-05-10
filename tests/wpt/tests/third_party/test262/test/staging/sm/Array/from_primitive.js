/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
for (let primitive of [true, 3.14, "hello", Symbol()]) {
    let prototype = Object.getPrototypeOf(primitive);

    Object.defineProperty(prototype, Symbol.iterator, {
        configurable: true,
        get() {
            "use strict";
            assert.sameValue(this, primitive);
            return () => [this][Symbol.iterator]();
        },
    });
    assert.sameValue(Array.from(primitive)[0], primitive);

    delete prototype[Symbol.iterator];
}

