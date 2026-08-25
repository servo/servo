// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-class-body-ctor-duplicate.case
// - src/class-elements/syntax/invalid/cls-decl-elements-invalid-syntax.template
/*---
description: It is a SyntaxError if the class body has more than one constructor (class declaration)
esid: prod-ClassElement
features: [class]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    ClassBody : ClassElementList
        It is a Syntax Error if PrototypePropertyNameList of ClassElementList contains more
        than one occurrence of "constructor".

---*/


$DONOTEVALUATE();

class C {
  constructor() {}
  constructor() {}
}
