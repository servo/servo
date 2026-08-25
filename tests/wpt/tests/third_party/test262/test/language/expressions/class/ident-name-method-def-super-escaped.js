// This file was procedurally generated from the following sources:
// - src/identifier-names/super-escaped.case
// - src/identifier-names/default/class-expression-method-def.template
/*---
description: super is a valid identifier name, using escape (MethodDefinition)
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
  sup\u0065r() { return 42; }
}

var obj = new C();

assert.sameValue(obj['super'](), 42, 'property exists');
