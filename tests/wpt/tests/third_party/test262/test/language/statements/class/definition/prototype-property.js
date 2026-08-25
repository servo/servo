// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    class prototype property
---*/
class C {}
var descr = Object.getOwnPropertyDescriptor(C, 'prototype');
assert.sameValue(descr.configurable, false, "The value of `descr.configurable` is `false`");
assert.sameValue(descr.enumerable, false, "The value of `descr.enumerable` is `false`");
assert.sameValue(descr.writable, false, "The value of `descr.writable` is `false`");
