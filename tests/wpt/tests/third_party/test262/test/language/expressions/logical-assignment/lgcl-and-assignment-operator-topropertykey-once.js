// Copyright (C) 2026 Sergey Rubanov. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assignment-operators-runtime-semantics-evaluation
description: >
  ToPropertyKey is called only once on the key of base[prop] &&= value when it assigns.
info: |
  AssignmentExpression : LeftHandSideExpression &&= AssignmentExpression

  1. Let leftRef be ? Evaluation of LeftHandSideExpression.
  2. Let leftValue be ? GetValue(leftRef).
  3. If ToBoolean(leftValue) is false, return leftValue.
  ...
  6. Perform ? PutValue(leftRef, rightValue).

  GetValue ( refRecord )

  ...
  3. If IsPropertyReference(refRecord) is true, then
    ...
    c. If refRecord.[[ReferencedName]] is not a property key, then
      i. Set refRecord.[[ReferencedName]] to ? ToPropertyKey(refRecord.[[ReferencedName]]).

  PutValue ( refRecord, value )

  ...
  3. If IsPropertyReference(refRecord) is true, then
    ...
    c. If refRecord.[[ReferencedName]] is not a property key, then
      i. Set refRecord.[[ReferencedName]] to ? ToPropertyKey(refRecord.[[ReferencedName]]).
features: [logical-assignment-operators]
---*/

var count = 0;
var prop = {
  toString: function() {
    count++;
    return "p";
  }
};
var base = { p: 1 };

base[prop] &&= 2;

assert.sameValue(count, 1, "ToPropertyKey(prop) is performed once");
assert.sameValue(base.p, 2, "the value is assigned to the converted key");
