// This file was procedurally generated from the following sources:
// - src/identifier-names/protected-escaped.case
// - src/identifier-names/future-reserved-words/member-expr.template
/*---
description: protected is a valid identifier name, using escape (MemberExpression IdentifierName)
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

obj.prot\u0065cted = 42;

assert.sameValue(obj['protected'], 42, 'property exists');
