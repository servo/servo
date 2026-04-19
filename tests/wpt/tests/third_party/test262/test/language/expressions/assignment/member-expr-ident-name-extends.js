// This file was procedurally generated from the following sources:
// - src/identifier-names/extends.case
// - src/identifier-names/default/member-expr.template
/*---
description: extends is a valid identifier name (MemberExpression IdentifierName)
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

obj.extends = 42;

assert.sameValue(obj['extends'], 42, 'property exists');
