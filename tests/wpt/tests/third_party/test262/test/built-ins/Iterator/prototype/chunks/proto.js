// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.chunks
description: >
  The value of the [[Prototype]] internal slot of Iterator.prototype.chunks is the
  intrinsic object %FunctionPrototype%.
features: [iterator-chunking]
---*/

assert.sameValue(Object.getPrototypeOf(Iterator.prototype.chunks), Function.prototype);
