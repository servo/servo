// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-__proto__-property-names-in-object-initializers
es6id: B.3.1
description: Duplicate `__proto__` property
info: |
    It is a Syntax Error if PropertyNameList of PropertyDefinitionList contains
    any duplicate entries for "__proto__" and at least two of those entries
    were obtained from productions of the form
    PropertyDefinition : PropertyName : AssignmentExpression .
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

({
  __proto__: null,
  other: null,
  '__proto__': null
});
