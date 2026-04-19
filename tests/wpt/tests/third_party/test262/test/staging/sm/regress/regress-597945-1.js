/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
var arguments;

function b(foo) {
    delete foo.d
    delete foo.w
    foo.d = true
    foo.w = Object
    delete Object.defineProperty(foo, "d", ({
        set: Math.w
    })); {}
}
for(e of [arguments, arguments]) {
    try {
        b(e)('')
    } catch (e) {}
}

