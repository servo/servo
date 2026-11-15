// Copyright (C) 2026 Sergey Rubanov. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assignment-operators-runtime-semantics-evaluation
description: >
  ToPropertyKey is called only once on the key of base[prop] **= value.
info: |
  AssignmentExpression : LeftHandSideExpression AssignmentOperator AssignmentExpression

  1. Let leftRef be ? Evaluation of LeftHandSideExpression.
  ...
  3. Let leftValue be ? GetValue(leftRef).
  ...
  9. Perform ? PutValue(leftRef, result).

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
features: [exponentiation]
---*/

var count = 0;
var prop = {
  toString: function() {
    count++;
    return "p";
  }
};
var base = { p: 2 };

base[prop] **= 3;

assert.sameValue(count, 1, "ToPropertyKey(prop) is performed once");
assert.sameValue(base.p, 8, "the result is assigned to the converted key");
