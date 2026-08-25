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
// getOwnPropertySymbols(proxy) calls the getOwnPropertyNames hook (only).

var symbols = [Symbol(), Symbol("moon"), Symbol.for("sun"), Symbol.iterator];
var hits = 0;

function HandlerProxy() {
    return new Proxy({}, {
        get: function (t, key) {
            if (key !== "ownKeys")
                throw new Error("tried to access handler[" + String(key) + "]");
            hits++;
            return t => symbols;
        }
    });
}

function OwnKeysProxy() {
    return new Proxy({}, new HandlerProxy);
}

assert.compareArray(Object.getOwnPropertySymbols(new OwnKeysProxy), symbols);
assert.sameValue(hits, 1);

