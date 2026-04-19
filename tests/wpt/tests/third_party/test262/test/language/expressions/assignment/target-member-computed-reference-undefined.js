// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assignment-operators
description: Assignment Operator evaluates the value prior validating a MemberExpression's reference (undefined)
info: |
  # 13.15.2 Runtime Semantics: Evaluation
  AssignmentExpression : LeftHandSideExpression = AssignmentExpression

  1. If LeftHandSideExpression is neither an ObjectLiteral nor an ArrayLiteral,
     then
     a. Let lref be the result of evaluating LeftHandSideExpression.
     [...]
     e. Perform ? PutValue(lref, rval).

  # 6.2.4.5 PutValue ( V, W )

  [...]
  5. If IsPropertyReference(V) is true, then
     a. Let baseObj be ? ToObject(V.[[Base]]).
---*/

function DummyError() { }

assert.throws(DummyError, function() {
  var base = undefined;
  var prop = function() {
    throw new DummyError();
  };
  var expr = function() {
    throw new Test262Error("right-hand side expression evaluated");
  };

  base[prop()] = expr();
});

assert.throws(DummyError, function() {
  var base = undefined;
  var prop = {
    toString: function() {
      throw new Test262Error("property key evaluated");
    }
  };
  var expr = function() {
    throw new DummyError();
  };

  base[prop] = expr();
});
