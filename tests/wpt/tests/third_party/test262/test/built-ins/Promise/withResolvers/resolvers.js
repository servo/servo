// Copyright (C) 2023 Peter Klecha. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Promise.withResolvers return value has properties called "resolve" and "reject" which are unary functions
esid: sec-promise.withresolvers
features: [promise-with-resolvers]
---*/


var instance = Promise.withResolvers();

assert.sameValue(typeof instance.resolve, 'function', 'type of resolve property');
assert.sameValue(instance.resolve.name, "");
assert.sameValue(instance.resolve.length, 1, 'length of resolve property');
assert.sameValue(typeof instance.reject, 'function', 'type of reject property');
assert.sameValue(instance.reject.name, "");
assert.sameValue(instance.reject.length, 1, 'length of reject property');
