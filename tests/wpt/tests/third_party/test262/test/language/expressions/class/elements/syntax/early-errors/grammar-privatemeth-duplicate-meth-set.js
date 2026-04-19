// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-privatemeth-duplicate-meth-set.case
// - src/class-elements/syntax/invalid/cls-expr-elements-invalid-syntax.template
/*---
description: It's a SyntaxError if a class contains a private method and a private setter with the same name (class expression)
esid: prod-ClassElement
features: [class-methods-private, class]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Static Semantics: Early Errors

    ClassBody : ClassElementList
        It is a Syntax Error if PrivateBoundNames of ClassBody contains any duplicate entries, unless the name is used once for a getter and once for a setter and in no other entries.

---*/


$DONOTEVALUATE();

var C = class {
  set #m(_) {}
  #m() {}
};
