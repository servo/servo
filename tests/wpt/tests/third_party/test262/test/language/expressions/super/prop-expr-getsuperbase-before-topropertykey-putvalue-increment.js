// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-super-keyword-runtime-semantics-evaluation
description: >
  GetSuperBase is performed before ToPropertyKey in PutValue with increment operator.
info: |
  13.3.7.1 Runtime Semantics: Evaluation

    SuperProperty : super [ Expression ]

    ...
    2. Let actualThis be ? env.GetThisBinding().
    3. Let propertyNameReference be ? Evaluation of Expression.
    4. Let propertyNameValue be ? GetValue(propertyNameReference).
    ...
    7. Return ? MakeSuperPropertyReference(actualThis, propertyNameValue, strict).

  13.3.7.3 MakeSuperPropertyReference ( actualThis, propertyKey, strict )

    1. Let env be GetThisEnvironment().
    ...
    3. Let baseValue be ? env.GetSuperBase().
    ...

  6.2.5.6 PutValue ( V, W )

    3. If IsPropertyReference(V) is true, then
      ...
      c. If V.[[ReferencedName]] is not a property key, then
          i. Set V.[[ReferencedName]] to ? ToPropertyKey(V.[[ReferencedName]]).
      d. Let succeeded be ? baseObj.[[Set]](V.[[ReferencedName]], W, GetThisValue(V)).
      ...
    ...
---*/

var proto = {
  p: 1,
};

var proto2 = {
  p: -1,
};

var obj = {
  __proto__: proto,
  m() {
    return ++super[key];
  }
};

var key = {
  toString() {
    Object.setPrototypeOf(obj, proto2);
    return "p";
  }
};

assert.sameValue(obj.m(), 2);
