// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-function-expression.case
// - src/computed-property-names/evaluation/object-literal.template
/*---
description: Computed property name from function expression (ComputedPropertyName in ObjectLiteral)
esid: prod-ComputedPropertyName
features: [computed-property-names]
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
  [function () {}]: 1
};

assert.sameValue(
  o[function () {}],
  1
);
assert.sameValue(
  o[String(function () {})],
  1
);
