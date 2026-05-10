// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%arrayiteratorprototype%-@@tostringtag
description: >
  The @@toStringTag attribute should be defined directly on the prototype.
es6id: 22.1.5.2.2
features: [Symbol.iterator, Symbol.toStringTag]
---*/

var ArrayIteratorProto = Object.getPrototypeOf([][Symbol.iterator]());

assert.sameValue("Array Iterator", ArrayIteratorProto[Symbol.toStringTag]);
