// Copyright (C) 2023 Peter Klecha. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Promise.withResolvers result is an object with keys "promise", "reject", and "resolve"
esid: sec-promise.withresolvers
includes: [propertyHelper.js]
features: [promise-with-resolvers]
---*/


var instance = Promise.withResolvers();

assert.sameValue(typeof instance, "object");
assert.notSameValue(instance, null);
assert(instance instanceof Object);

verifyProperty(instance, "promise", {
    writable: true,
    configurable: true,
    enumerable: true,
})

verifyProperty(instance, "resolve", {
    writable: true,
    configurable: true,
    enumerable: true,
})

verifyProperty(instance, "reject", {
    writable: true,
    configurable: true,
    enumerable: true,
})
