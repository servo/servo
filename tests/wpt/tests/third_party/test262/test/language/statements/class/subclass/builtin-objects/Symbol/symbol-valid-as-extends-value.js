// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.4.1
description: Symbol can be used as the value of an extends
info: |
  19.4.1 The Symbol Constructor

  ...
  The Symbol constructor is not intended to be used with the new operator or to
  be subclassed. It may be used as the value of an extends clause of a class
  definition but a super call to the Symbol constructor will cause an exception.
  ...
features: [Symbol]
---*/

class S extends Symbol {}
