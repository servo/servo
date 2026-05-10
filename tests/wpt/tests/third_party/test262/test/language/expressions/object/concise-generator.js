// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.5
description: >
    super method calls in object literal concise generator
features: [generators]
---*/
var proto = {
  method() {
    return 42;
  }
};

var object = {
  *g() {
    yield super.method();
  }
};

Object.setPrototypeOf(object, proto);

assert.sameValue(
  object.g().next().value,
  42,
  "The value of `object.g().next().value` is `42`, after executing `Object.setPrototypeOf(object, proto);`, where `object " + String(object) + "`"
);
