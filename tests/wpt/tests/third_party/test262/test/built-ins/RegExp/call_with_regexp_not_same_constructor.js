// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: RegExp returns a new object if constructor is not same-value
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
---*/

var regExpObj = /(?:)/;
regExpObj.constructor = null;

assert.notSameValue(RegExp(regExpObj), regExpObj, "RegExp returns new object");
