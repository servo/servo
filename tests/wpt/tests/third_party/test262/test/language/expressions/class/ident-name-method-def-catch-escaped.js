// This file was procedurally generated from the following sources:
// - src/identifier-names/catch-escaped.case
// - src/identifier-names/default/class-expression-method-def.template
/*---
description: catch is a valid identifier name, using escape (MethodDefinition)
esid: prod-PropertyDefinition
features: [class]
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


var C = class {
  c\u0061tch() { return 42; }
}

var obj = new C();

assert.sameValue(obj['catch'](), 42, 'property exists');
