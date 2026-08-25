// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.5.6.1.1
description: >
  The name property on a new instance
info: |
  19.5.6.3.3 NativeError.prototype.name

  The initial value of the name property of the prototype for a given
  NativeError constructor is a string consisting of the name of the constructor
  (the name used instead of NativeError).
---*/

class Err extends SyntaxError {}

var err1 = new Err();
assert.sameValue(err1.name, 'SyntaxError');
