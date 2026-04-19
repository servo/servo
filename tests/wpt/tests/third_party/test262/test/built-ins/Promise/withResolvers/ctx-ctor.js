// Copyright (C) 2023 Peter Klecha. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Promise.withResolvers produces instances of the receiver
esid: sec-promise.withresolvers
features: [promise-with-resolvers, class]
---*/

class SubPromise extends Promise {}

var instance = Promise.withResolvers.call(SubPromise);

assert.sameValue(instance.promise.constructor, SubPromise);
assert.sameValue(instance.promise instanceof SubPromise, true);

