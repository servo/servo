// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-set-p-v-receiver
description: >
    [[Set]] ( P, V, Receiver)

    7. If trap is undefined, then
      a. Return ? target.[[Set]](P, V, Receiver)

features: [Proxy]
---*/

var target = {
  attr: 1
};
var p = new Proxy(target, {
  set: undefined
});

p.attr = 2;

assert.sameValue(target.attr, 2);
