// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-integer-separators.case
// - src/computed-property-names/evaluation/class-declaration.template
/*---
description: Computed property name from integer with separators (ComputedPropertyName in ClassDeclaration)
esid: prod-ComputedPropertyName
features: [computed-property-names]
flags: [generated]
info: |
    ClassExpression:
      classBindingIdentifier opt ClassTail

    ClassTail:
      ClassHeritage opt { ClassBody opt }

    ClassBody:
      ClassElementList

    ClassElementList:
      ClassElement

    ClassElement:
      MethodDefinition

    MethodDefinition:
      PropertyName ...
      get PropertyName ...
      set PropertyName ...

    PropertyName:
      ComputedPropertyName

    ComputedPropertyName:
      [ AssignmentExpression ]
---*/


class C {
  [1_2_3_4_5_6_7_8]() {
    return 1_2_3_4_5_6_7_8;
  }
  static [1_2_3_4_5_6_7_8]() {
    return 1_2_3_4_5_6_7_8;
  }
};

let c = new C();

assert.sameValue(
  c[1_2_3_4_5_6_7_8](),
  1_2_3_4_5_6_7_8
);
assert.sameValue(
  C[1_2_3_4_5_6_7_8](),
  1_2_3_4_5_6_7_8
);
assert.sameValue(
  c[String(1_2_3_4_5_6_7_8)](),
  1_2_3_4_5_6_7_8
);
assert.sameValue(
  C[String(1_2_3_4_5_6_7_8)](),
  1_2_3_4_5_6_7_8
);
