// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-ownpropertykeys
description: >
    If target is not extensible, the result can't contain keys names not
    contained in the target object.
info: |
    [[OwnPropertyKeys]] ( )

    ...
    20. If uncheckedResultKeys is not empty, throw a TypeError exception.
features: [Proxy]
---*/

var target = {
  foo: 1
};

var p = new Proxy(target, {
  ownKeys: function() {
    return ["foo", "bar"];
  }
});

Object.preventExtensions(target);

assert.throws(TypeError, function() {
  Object.keys(p);
});
