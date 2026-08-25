// This file was procedurally generated from the following sources:
// - src/identifier-names/delete-escaped.case
// - src/identifier-names/default/member-expr.template
/*---
description: delete is a valid identifier name, using escape (MemberExpression IdentifierName)
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

obj.\u0064elete = 42;

assert.sameValue(obj['delete'], 42, 'property exists');
