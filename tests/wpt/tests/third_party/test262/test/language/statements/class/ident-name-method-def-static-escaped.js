// This file was procedurally generated from the following sources:
// - src/identifier-names/static-escaped.case
// - src/identifier-names/future-reserved-words/class-statement-method-def.template
/*---
description: static is a valid identifier name, using escape (MethodDefinition)
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


class C {
  st\u0061tic() { return 42; }
}

var obj = new C();

assert.sameValue(obj['static'](), 42, 'property exists');
