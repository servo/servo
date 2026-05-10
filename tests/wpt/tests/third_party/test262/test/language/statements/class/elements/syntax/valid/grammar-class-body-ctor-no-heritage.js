// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-class-body-ctor-no-heritage.case
// - src/class-elements/syntax/valid/cls-decl-elements-valid-syntax.template
/*---
description: A constructor is valid without a super call in the constructor and heritage (class declaration)
esid: prod-ClassElement
features: [class]
flags: [generated]
info: |
    ClassTail : ClassHeritageopt { ClassBody }

    It is a Syntax Error if ClassHeritage is not present and the following algorithm evaluates to true:
      1. Let constructor be ConstructorMethod of ClassBody.
      2. If constructor is empty, return false.
      3. Return HasDirectSuper of constructor.

---*/


class C {
  constructor() {}
}
