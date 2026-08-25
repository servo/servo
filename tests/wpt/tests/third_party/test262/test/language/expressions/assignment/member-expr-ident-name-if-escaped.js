// This file was procedurally generated from the following sources:
// - src/identifier-names/if-escaped.case
// - src/identifier-names/default/member-expr.template
/*---
description: if is a valid identifier name, using escape (MemberExpression IdentifierName)
esid: prod-PropertyDefinition
flags: [generated]
info: |
    MemberExpression:
      ...
      MemberExpression . IdentifierName

    Reserved Words

    A reserved word is an IdentifierName that cannot be used as an Identifier.
---*/

var obj = {};

obj.i\u0066 = 42;

assert.sameValue(obj['if'], 42, 'property exists');
