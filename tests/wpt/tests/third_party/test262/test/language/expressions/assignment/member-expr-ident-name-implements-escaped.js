// This file was procedurally generated from the following sources:
// - src/identifier-names/implements-escaped.case
// - src/identifier-names/future-reserved-words/member-expr.template
/*---
description: implements is a valid identifier name, using escape (MemberExpression IdentifierName)
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

obj.\u0069mplements = 42;

assert.sameValue(obj['implements'], 42, 'property exists');
