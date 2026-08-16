// Copyright (C) 2025 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.prototype.join
description: Iterator.prototype.join is not a constructor
includes: [isConstructor.js]
features: [Iterator.prototype.join, Reflect.construct]
---*/

assert(!isConstructor(Iterator.prototype.join), "Iterator.prototype.join should not be a constructor");

assert.throws(TypeError, function() {
  var iterator = [].values();
  new iterator.join();
});
