// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-special-meth-ctor-async-gen.case
// - src/class-elements/syntax/invalid/cls-expr-elements-invalid-syntax.template
/*---
description: Async Generator Methods cannot be named "constructor" (class expression)
esid: prod-ClassElement
features: [async-iteration, class]
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
  async * constructor() {}
};
