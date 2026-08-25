// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object-value
author: Matthew Phillips <matthew@matthewphillips.info>
description: >
  NewTarget is active function and subclass of Object
info: |
  Object ( [ value ] )

  1. If NewTarget is neither undefined nor the active function, then
    a. Return ? OrdinaryCreateFromConstructor(NewTarget, "%ObjectPrototype%").
  [...]
features: [class, Reflect, Reflect.construct]
---*/

class O extends Object {}

var o1 = new O({a: 1});
var o2 = Reflect.construct(Object, [{b: 2}], O);

assert.sameValue(o1.a, undefined);
assert.sameValue(o2.b, undefined);

assert.sameValue(Object.getPrototypeOf(o1), O.prototype);
assert.sameValue(Object.getPrototypeOf(o2), O.prototype);
