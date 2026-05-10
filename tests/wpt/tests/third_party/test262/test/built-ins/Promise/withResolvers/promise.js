// Copyright (C) 2023 Peter Klecha. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Promise.withResolvers return value has a property called "promise" which is a Promise
esid: sec-promise.withresolvers
features: [promise-with-resolvers]
---*/


var instance = Promise.withResolvers();

assert.sameValue(instance.promise.constructor, Promise);
assert.sameValue(instance.promise instanceof Promise, true);

