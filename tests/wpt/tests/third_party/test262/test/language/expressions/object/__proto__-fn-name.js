// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-semantics-propertydefinitionevaluation
description: Function name is not assigned based on the __proto__ property name
info: |
    [...]
    3. Else if propKey is "__proto__" and IsComputedPropertyKey of PropertyName is false, then
        a. Let isProtoSetter be true.
    [...]
    5. If IsAnonymousFunctionDefinition(AssignmentExpression) is true and isProtoSetter is false, then
        a. Let propValue be ? NamedEvaluation of AssignmentExpression with argument propKey.
    6. Else,
        a. Let exprValueRef be ? Evaluation of AssignmentExpression.
---*/

var o;

o = {
  __proto__: function () {},
};

assert(Object.getPrototypeOf(o).name !== "__proto__");
