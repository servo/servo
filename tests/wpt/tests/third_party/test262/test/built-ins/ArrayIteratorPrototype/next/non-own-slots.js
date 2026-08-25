// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    If the `this` value does not have all of the internal slots of an Array
    Iterator Instance (22.1.5.3), throw a TypeError exception.
esid: sec-%arrayiteratorprototype%.next
features: [Symbol.iterator]
---*/

var array = [0];
var iterator = array[Symbol.iterator]();
var object = Object.create(iterator);

assert.throws(TypeError, function() {
  object.next();
});
