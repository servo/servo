// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x++ evaluates its reference expression once.
description: >
    The operand expression is evaluated exactly once. Operand expression is
    MemberExpression: base[prop]. base is the undefined value.
---*/

function DummyError() { }

assert.throws(DummyError, function() {
  var base = undefined;
  var prop = function() {
    throw new DummyError();
  };

  base[prop()]++;
});

assert.throws(TypeError, function() {
  var base = undefined;
  var prop = {
    toString: function() {
      throw new Test262Error("property key evaluated");
    }
  };

  base[prop]++;
});
