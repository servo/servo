// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-special-meth-ctor-get.case
// - src/class-elements/syntax/invalid/cls-expr-elements-invalid-syntax.template
/*---
description: Accessor get Methods cannot be named "constructor" (class expression)
esid: prod-ClassElement
features: [class]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Class Definitions / Static Semantics: Early Errors

    ClassElement : MethodDefinition
        It is a Syntax Error if PropName of MethodDefinition is "constructor" and SpecialMethod of MethodDefinition is true.

---*/


$DONOTEVALUATE();

var C = class {
  get constructor() {}
};
