// This file was procedurally generated from the following sources:
// - src/identifier-names/default-escaped-ext.case
// - src/identifier-names/default/obj-method-definition.template
/*---
description: default is a valid identifier name, using extended escape (MethodDefinition)
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
  def\u{61}ult() { return 42; }
};

assert.sameValue(obj['default'](), 42, 'property exists');
