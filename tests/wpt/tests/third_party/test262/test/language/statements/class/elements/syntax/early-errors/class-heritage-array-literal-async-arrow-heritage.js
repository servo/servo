// This file was procedurally generated from the following sources:
// - src/class-elements/class-heritage-array-literal-async-arrow-heritage.case
// - src/class-elements/syntax/invalid/cls-decl-elements-invalid-syntax.template
/*---
description: It's a SyntaxError if an array literal evaluated on ClassHeritage uses a private name. (class declaration)
esid: prod-ClassElement
features: [class]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    ClassTail : ClassHeritage { ClassBody }

    ClassHeritage :
        extends LeftHandSideExpression

---*/


$DONOTEVALUATE();

class C extends async () => {} {
  
}
