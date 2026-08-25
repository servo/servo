// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-math.case
// - src/computed-property-names/evaluation/object-literal.template
/*---
description: Computed property name from math (ComputedPropertyName in ObjectLiteral)
esid: prod-ComputedPropertyName
features: [computed-property-names, exponentiation]
flags: [generated]
info: |
    ObjectLiteral:
      { PropertyDefinitionList }

    PropertyDefinitionList:
      PropertyDefinition

    PropertyDefinition:
      PropertyName: AssignmentExpression

    PropertyName:
      ComputedPropertyName

    ComputedPropertyName:
      [ AssignmentExpression ]
---*/


let o = {
  [1 + 2 - 3 * 4 / 5 ** 6]: 2.999232
};

assert.sameValue(
  o[1 + 2 - 3 * 4 / 5 ** 6],
  2.999232
);
assert.sameValue(
  o[String(1 + 2 - 3 * 4 / 5 ** 6)],
  2.999232
);
