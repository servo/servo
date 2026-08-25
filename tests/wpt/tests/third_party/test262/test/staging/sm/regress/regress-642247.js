/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
if (typeof timeout == "function") {
    assert.sameValue(typeof timeout(), "number");
    assert.sameValue(typeof timeout(1), "undefined");
}

