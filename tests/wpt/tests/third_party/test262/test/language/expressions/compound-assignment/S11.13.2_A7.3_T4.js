// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Compound Assignment Operator evaluates its operands from left to right.
description: >
    The left-hand side expression is evaluated before the right-hand side.
    Left-hand side expression is MemberExpression: base[prop]. ToPropertyKey(prop)
    is only called once.
    Check operator is "x %= y".
---*/

var propKeyEvaluated = false;
var base = {};
var prop = {
  toString: function() {
    assert(!propKeyEvaluated);
    propKeyEvaluated = true;
    return "";
  }
};
var expr = function() {
  return 0;
};

base[prop] %= expr();
