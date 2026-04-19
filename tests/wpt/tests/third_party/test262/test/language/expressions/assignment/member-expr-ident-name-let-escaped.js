// This file was procedurally generated from the following sources:
// - src/identifier-names/let-escaped.case
// - src/identifier-names/future-reserved-words/member-expr.template
/*---
description: let is a valid identifier name, using escape (MemberExpression IdentifierName)
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

obj.l\u0065t = 42;

assert.sameValue(obj['let'], 42, 'property exists');
