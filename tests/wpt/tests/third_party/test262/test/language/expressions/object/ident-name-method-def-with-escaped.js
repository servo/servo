// This file was procedurally generated from the following sources:
// - src/identifier-names/with-escaped.case
// - src/identifier-names/default/obj-method-definition.template
/*---
description: with is a valid identifier name, using escape (MethodDefinition)
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
  w\u0069th() { return 42; }
};

assert.sameValue(obj['with'](), 42, 'property exists');
