// Copyright (C) 2023 Peter Klecha. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Promise.withResolvers errors when the receiver is not a constructor
esid: sec-promise.withresolvers
features: [promise-with-resolvers]
---*/

assert.throws(TypeError, function() {
  Promise.withResolvers.call(eval);
});
