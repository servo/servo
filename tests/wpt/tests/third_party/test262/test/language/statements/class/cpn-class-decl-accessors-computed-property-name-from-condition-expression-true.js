// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-condition-expression-true.case
// - src/computed-property-names/evaluation/class-declaration-accessors.template
/*---
description: Computed property name from condition expression (ComputedPropertyName in ClassDeclaration)
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
  get [true ? 1 : 2]() {
    return 2;
  }

  set [true ? 1 : 2](v) {
    return 2;
  }

  static get [true ? 1 : 2]() {
    return 2;
  }

  static set [true ? 1 : 2](v) {
    return 2;
  }
};

let c = new C();

assert.sameValue(
  c[true ? 1 : 2],
  2
);
assert.sameValue(
  c[true ? 1 : 2] = 2,
  2
);

assert.sameValue(
  C[true ? 1 : 2],
  2
);
assert.sameValue(
  C[true ? 1 : 2] = 2,
  2
);
assert.sameValue(
  c[String(true ? 1 : 2)],
  2
);
assert.sameValue(
  c[String(true ? 1 : 2)] = 2,
  2
);

assert.sameValue(
  C[String(true ? 1 : 2)],
  2
);
assert.sameValue(
  C[String(true ? 1 : 2)] = 2,
  2
);
