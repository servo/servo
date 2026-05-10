// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-static-get-meth-prototype.case
// - src/class-elements/syntax/invalid/cls-decl-elements-invalid-syntax.template
/*---
description: Static Accessor get Methods cannot be named prototype (class declaration)
esid: prod-ClassElement
features: [class]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Class Definitions / Static Semantics: Early Errors

    ClassElement : static MethodDefinition
        It is a Syntax Error if PropName of MethodDefinition is "prototype"

---*/


$DONOTEVALUATE();

class C {
  static get prototype() {}
}
