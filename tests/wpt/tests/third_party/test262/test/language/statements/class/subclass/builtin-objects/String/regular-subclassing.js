// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.1
description: Subclassing the String object
info: |
  21.1.1 The String Constructor

  ...
  The String constructor is designed to be subclassable. It may be used as the
  value of an extends clause of a class definition. Subclass constructors that
  intend to inherit the specified String behaviour must include a super call to
  the String constructor to create and initialize the subclass instance with a
  [[StringData]] internal slot.
---*/

class S extends String {}

var s = new S(' test262 ');

assert.sameValue(s.trim(), 'test262');

