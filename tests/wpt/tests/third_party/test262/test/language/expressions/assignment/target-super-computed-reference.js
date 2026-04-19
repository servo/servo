// Copyright (C) 2024 Sony Interactive Entertainment Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assignment-operators
description: Assignment Operator evaluates its operands from left to right
info: |
  The left-hand side expression is evaluated before the right-hand side.
  Left-hand side expression is MemberExpression: super[prop].
  ToPropertyKey(prop) occurs after both sides are evaluated.
---*/

function DummyError() {}

assert.throws(DummyError, function() {
  var prop = function() {
    throw new DummyError();
  };
  var expr = function() {
    throw new Test262Error("right-hand side expression evaluated");
  };

  class C extends class {} {
    m() {
      super[prop()] = expr();
    }
  }
  
  (new C()).m();
});

assert.throws(DummyError, function() {
  var prop = {
    toString: function() {
      throw new Test262Error("property key evaluated");
    }
  };
  var expr = function() {
    throw new DummyError();
  };

  class C extends class {} {
    m() {
      super[prop] = expr();
    }
  }
  
  (new C()).m();
});
