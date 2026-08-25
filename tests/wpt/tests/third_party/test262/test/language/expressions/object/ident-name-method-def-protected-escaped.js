// This file was procedurally generated from the following sources:
// - src/identifier-names/protected-escaped.case
// - src/identifier-names/future-reserved-words/obj-method-definition.template
/*---
description: protected is a valid identifier name, using escape (MethodDefinition)
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
      MethodDefinition
      ...

    MethodDefinition:
      PropertyName ( UniqueFormalParameters ){ FunctionBody }

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
  prot\u0065cted() { return 42; }
};

assert.sameValue(obj['protected'](), 42, 'property exists');
