// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.8
description: >
    [[Get]] (P, Receiver)

    8. If trap is undefined, then return target.[[Get]](P, Receiver).
features: [Proxy]
---*/

var target = {
  attr: 1
};
var p = new Proxy(target, {});

assert.sameValue(p.attr, 1, 'return target.attr');
assert.sameValue(p.foo, undefined, 'return target.foo');
assert.sameValue(p['attr'], 1, 'return target.attr');
assert.sameValue(p['foo'], undefined, 'return target.foo');
