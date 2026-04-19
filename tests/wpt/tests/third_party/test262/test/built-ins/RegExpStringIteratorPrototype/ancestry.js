// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: |
    The [[Prototype]] internal slot ofthe %RegExpStringIteratorPrototype% is the
    %IteratorPrototype% intrinsic object (25.1.2).
features: [Symbol.iterator, Symbol.matchAll]
---*/

var RegExpStringIteratorProto = Object.getPrototypeOf(/./[Symbol.matchAll]('a'));
var ArrayIteratorProto = Object.getPrototypeOf(
  Object.getPrototypeOf([][Symbol.iterator]())
);

assert.sameValue(Object.getPrototypeOf(RegExpStringIteratorProto), ArrayIteratorProto);
