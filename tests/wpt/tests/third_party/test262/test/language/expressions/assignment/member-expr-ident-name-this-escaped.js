// This file was procedurally generated from the following sources:
// - src/identifier-names/this-escaped.case
// - src/identifier-names/default/member-expr.template
/*---
description: this is a valid identifier name, using escape (MemberExpression IdentifierName)
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

obj.th\u0069s = 42;

assert.sameValue(obj['this'], 42, 'property exists');
