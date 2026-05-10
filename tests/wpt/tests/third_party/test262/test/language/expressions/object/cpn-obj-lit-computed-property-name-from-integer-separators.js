// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-integer-separators.case
// - src/computed-property-names/evaluation/object-literal.template
/*---
description: Computed property name from integer with separators (ComputedPropertyName in ObjectLiteral)
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
  [1_2_3_4_5_6_7_8]: 1_2_3_4_5_6_7_8
};

assert.sameValue(
  o[1_2_3_4_5_6_7_8],
  1_2_3_4_5_6_7_8
);
assert.sameValue(
  o[String(1_2_3_4_5_6_7_8)],
  1_2_3_4_5_6_7_8
);
