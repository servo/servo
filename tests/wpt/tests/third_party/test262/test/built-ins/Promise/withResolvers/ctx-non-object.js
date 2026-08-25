// Copyright (C) 2023 Peter Klecha. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Promise.withResolvers errors when the receiver is not an object
esid: sec-promise.withresolvers
features: [promise-with-resolvers]
---*/

assert.throws(TypeError, function() {
  Promise.withResolvers.call(undefined);
});

assert.throws(TypeError, function() {
  Promise.withResolvers.call(null);
});

assert.throws(TypeError, function() {
  Promise.withResolvers.call(86);
});
  
assert.throws(TypeError, function() {
  Promise.withResolvers.call('string');
});

assert.throws(TypeError, function() {
  Promise.withResolvers.call(true);
});

assert.throws(TypeError, function() {
  Promise.withResolvers.call(Symbol());
});
