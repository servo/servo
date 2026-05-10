// This file was procedurally generated from the following sources:
// - src/identifier-names/default.case
// - src/identifier-names/default/member-expr.template
/*---
description: default is a valid identifier name (MemberExpression IdentifierName)
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

obj.default = 42;

assert.sameValue(obj['default'], 42, 'property exists');
