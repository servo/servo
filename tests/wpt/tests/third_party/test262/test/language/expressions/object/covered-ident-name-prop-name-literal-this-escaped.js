// This file was procedurally generated from the following sources:
// - src/identifier-names/this-escaped.case
// - src/identifier-names/default/covered-obj-prop-name.template
/*---
description: this is a valid identifier name, using escape (PropertyName in a CoverParenthesizedExpressionAndArrowParameterList)
esid: prod-PropertyDefinition
flags: [generated]
info: |
    ObjectLiteral :
      { PropertyDefinitionList }
      { PropertyDefinitionList , }

    PropertyDefinitionList:
      PropertyDefinition
      PropertyDefinitionList , PropertyDefinition

    PropertyDefinition:
      IdentifierReference
      PropertyName : AssignmentExpression
      MethodDefinition
      ... AssignmentExpression
      ...

    PropertyName:
      LiteralPropertyName
      ...

    LiteralPropertyName:
      IdentifierName
      ...

    Reserved Words

    A reserved word is an IdentifierName that cannot be used as an Identifier.
---*/

var obj = ({ th\u0069s: 42 });

assert.sameValue(obj['this'], 42, 'property exists');
