// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-math.case
// - src/computed-property-names/evaluation/class-declaration-accessors.template
/*---
description: Computed property name from math (ComputedPropertyName in ClassDeclaration)
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
  get [1 + 2 - 3 * 4 / 5 ** 6]() {
    return 2.999232;
  }

  set [1 + 2 - 3 * 4 / 5 ** 6](v) {
    return 2.999232;
  }

  static get [1 + 2 - 3 * 4 / 5 ** 6]() {
    return 2.999232;
  }

  static set [1 + 2 - 3 * 4 / 5 ** 6](v) {
    return 2.999232;
  }
};

let c = new C();

assert.sameValue(
  c[1 + 2 - 3 * 4 / 5 ** 6],
  2.999232
);
assert.sameValue(
  c[1 + 2 - 3 * 4 / 5 ** 6] = 2.999232,
  2.999232
);

assert.sameValue(
  C[1 + 2 - 3 * 4 / 5 ** 6],
  2.999232
);
assert.sameValue(
  C[1 + 2 - 3 * 4 / 5 ** 6] = 2.999232,
  2.999232
);
assert.sameValue(
  c[String(1 + 2 - 3 * 4 / 5 ** 6)],
  2.999232
);
assert.sameValue(
  c[String(1 + 2 - 3 * 4 / 5 ** 6)] = 2.999232,
  2.999232
);

assert.sameValue(
  C[String(1 + 2 - 3 * 4 / 5 ** 6)],
  2.999232
);
assert.sameValue(
  C[String(1 + 2 - 3 * 4 / 5 ** 6)] = 2.999232,
  2.999232
);
