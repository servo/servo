// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-additive-expression-subtract.case
// - src/computed-property-names/evaluation/class-declaration-accessors.template
/*---
description: Computed property name from additive expression "subtract" (ComputedPropertyName in ClassDeclaration)
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
  get [1 - 1]() {
    return 0;
  }

  set [1 - 1](v) {
    return 0;
  }

  static get [1 - 1]() {
    return 0;
  }

  static set [1 - 1](v) {
    return 0;
  }
};

let c = new C();

assert.sameValue(
  c[1 - 1],
  0
);
assert.sameValue(
  c[1 - 1] = 0,
  0
);

assert.sameValue(
  C[1 - 1],
  0
);
assert.sameValue(
  C[1 - 1] = 0,
  0
);
assert.sameValue(
  c[String(1 - 1)],
  0
);
assert.sameValue(
  c[String(1 - 1)] = 0,
  0
);

assert.sameValue(
  C[String(1 - 1)],
  0
);
assert.sameValue(
  C[String(1 - 1)] = 0,
  0
);
