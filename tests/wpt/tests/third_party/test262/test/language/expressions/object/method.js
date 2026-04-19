// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.5
description: >
    super method calls in object literal method
---*/
var proto = {
  method(x) {
    return 'proto' + x;
  }
};

var object = {
  method(x) {
    return super.method(x);
  }
};

Object.setPrototypeOf(object, proto);

assert.sameValue(object.method(42), 'proto42', "`object.method(42)` returns `'proto42'`, after executing `Object.setPrototypeOf(object, proto);`");
assert.sameValue(proto.method(42), 'proto42', "`proto.method(42)` returns `'proto42'`, after executing `Object.setPrototypeOf(object, proto);`");
assert.sameValue(
  Object.getPrototypeOf(object).method(42),
  'proto42',
  "`Object.getPrototypeOf(object).method(42)` returns `'proto42'`"
);
