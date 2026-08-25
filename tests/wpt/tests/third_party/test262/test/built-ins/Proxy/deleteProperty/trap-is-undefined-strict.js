// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.10
description: >
    [[Delete]] (P)

    8. If trap is undefined, then Return target.[[Delete]](P).
flags: [onlyStrict]
features: [Proxy, Reflect]
---*/

var target = {
  attr: 1
};
var p = new Proxy(target, {});

assert.sameValue(delete p.attr, true);
assert.sameValue(delete p.notThere, true);
assert.sameValue(
  Object.getOwnPropertyDescriptor(target, "attr"),
  undefined
);

Object.defineProperty(target, "attr", {
  configurable: false,
  enumerable: true,
  value: 1
});

assert.sameValue(Reflect.deleteProperty(p, "attr"), false);
