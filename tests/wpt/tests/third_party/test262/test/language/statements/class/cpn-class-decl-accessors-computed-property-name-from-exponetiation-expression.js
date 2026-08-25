// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-exponetiation-expression.case
// - src/computed-property-names/evaluation/class-declaration-accessors.template
/*---
description: Computed property name from exponentiation expression (ComputedPropertyName in ClassDeclaration)
esid: prod-ComputedPropertyName
features: [computed-property-names, exponentiation]
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
  get [2 ** 2]() {
    return 4;
  }

  set [2 ** 2](v) {
    return 4;
  }

  static get [2 ** 2]() {
    return 4;
  }

  static set [2 ** 2](v) {
    return 4;
  }
};

let c = new C();

assert.sameValue(
  c[2 ** 2],
  4
);
assert.sameValue(
  c[2 ** 2] = 4,
  4
);

assert.sameValue(
  C[2 ** 2],
  4
);
assert.sameValue(
  C[2 ** 2] = 4,
  4
);
assert.sameValue(
  c[String(2 ** 2)],
  4
);
assert.sameValue(
  c[String(2 ** 2)] = 4,
  4
);

assert.sameValue(
  C[String(2 ** 2)],
  4
);
assert.sameValue(
  C[String(2 ** 2)] = 4,
  4
);
