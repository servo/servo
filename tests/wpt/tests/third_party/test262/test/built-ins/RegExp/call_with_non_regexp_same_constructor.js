// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: RegExp returns its input argument if constructor is same-value
info: |
  21.2.3.1 RegExp ( pattern, flags )

  ...
  4. Else,
    a. Let newTarget be the active function object.
    b. If patternIsRegExp is true and flags is undefined, then
      i.   Let patternConstructor be Get(pattern, "constructor").
      ii.  ReturnIfAbrupt(patternConstructor).
      iii. If SameValue(newTarget, patternConstructor) is true, return pattern.
es6id: 21.2.3.1
features: [Symbol.match]
---*/

var obj = {
  constructor: RegExp
};
obj[Symbol.match] = true;

assert.sameValue(RegExp(obj), obj, "RegExp returns its input argument");
