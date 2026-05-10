// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.6
description: >
  Return value where property key is a symbol.
info: |
  26.1.6 Reflect.get ( target, propertyKey [ , receiver ])

  ...
  2. Let key be ToPropertyKey(propertyKey).
  ...

  7.1.14 ToPropertyKey ( argument )

  ...
  3. If Type(key) is Symbol, then
    a. Return key.
  ...
features: [Reflect, Symbol]
---*/

var o = {};
var s = Symbol('1');
o[s] = 42;

assert.sameValue(Reflect.get(o, s), 42);
