// Copyright (C) 2024 Sony Interactive Entertainment Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-evaluate-property-access-with-expression-key
description: When getting the value of o[p], ToObject(o) precedes ToPropertyKey(p).
info: |
  13.3.3 EvaluatePropertyAccessWithExpressionKey ( baseValue, expression, strict )
    1. Let _propertyNameReference_ be ? Evaluation of _expression_.
    2. Let _propertyNameValue_ be ? GetValue(_propertyNameReference_).
    ...
    4. Return the Reference Record { [[Base]]: _baseValue_, [[ReferencedName]]: _propertyNameValue_, [[Strict]]: _strict_, [[ThisValue]]: ~empty~ }.

  6.2.5.5 GetValue ( V )
    1. If V is not a Reference Record, return V.
    ...
    3. If IsPropertyReference(V) is true, then
      a. Let baseObj be ? ToObject(V.[[Base]]).
      ...
      c. If V.[[ReferencedName]] is neither a String nor a Symbol, then
        i. Let referencedName be ? ToPropertyKey(V.[[ReferencedName]]).
---*/

assert.throws(TypeError, function() {
  var base = null;
  var prop = {
    toString: function() {
      throw new Test262Error("property key evaluated");
    }
  };

  base[prop];
});

assert.throws(TypeError, function() {
  var base = undefined;
  var prop = {
    toString: function() {
      throw new Test262Error("property key evaluated");
    }
  };

  base[prop];
});
