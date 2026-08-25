// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.2
description: >
    When a generator function is invoked as a method of an object, its context
    is that object.
features: [generators]
---*/

var context;

function* g() {
  context = this;
}
var obj = {
  g: g
};
var iter = obj.g();

iter.next();

assert.sameValue(context, obj);
