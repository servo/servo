// This file was procedurally generated from the following sources:
// - src/identifier-names/void-escaped.case
// - src/identifier-names/default/obj-prop-name.template
/*---
description: void is a valid identifier name, using escape (PropertyName)
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

var obj = {
  voi\u0064: 42
};

assert.sameValue(obj['void'], 42, 'property exists');
