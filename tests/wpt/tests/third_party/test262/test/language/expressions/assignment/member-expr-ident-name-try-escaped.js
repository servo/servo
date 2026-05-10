// This file was procedurally generated from the following sources:
// - src/identifier-names/try-escaped.case
// - src/identifier-names/default/member-expr.template
/*---
description: try is a valid identifier name, using escape (MemberExpression IdentifierName)
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

obj.tr\u0079 = 42;

assert.sameValue(obj['try'], 42, 'property exists');
