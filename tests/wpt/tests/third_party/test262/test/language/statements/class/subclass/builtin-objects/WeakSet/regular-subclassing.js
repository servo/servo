// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 23.4.1
description: Subclassing the WeakSet object
info: |
  23.4.1 The WeakSet Constructor

  ...

  The WeakSet constructor is designed to be subclassable. It may be used as the
  value in an extends clause of a class definition. Subclass constructors that
  intend to inherit the specified WeakSet behaviour must include a super call to
  the WeakSet constructor to create and initialize the subclass instance with
  the internal state necessary to support the WeakSet.prototype built-in
  methods.
features: [WeakSet]
---*/

class WS extends WeakSet {}

var set = new WS();
var obj = {};

assert.sameValue(set.has(obj), false);

set.add(obj);
assert.sameValue(set.has(obj), true);
