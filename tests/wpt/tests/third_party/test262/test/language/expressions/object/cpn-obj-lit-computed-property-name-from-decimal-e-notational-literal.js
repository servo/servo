// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-decimal-e-notational-literal.case
// - src/computed-property-names/evaluation/object-literal.template
/*---
description: Computed property name from decimal e notational literal (ComputedPropertyName in ObjectLiteral)
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
  [1.e1]: 2
};

assert.sameValue(
  o[1.e1],
  2
);
assert.sameValue(
  o[String(1.e1)],
  2
);
