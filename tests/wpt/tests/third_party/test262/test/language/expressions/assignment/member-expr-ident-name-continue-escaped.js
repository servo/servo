// This file was procedurally generated from the following sources:
// - src/identifier-names/continue-escaped.case
// - src/identifier-names/default/member-expr.template
/*---
description: continue is a valid identifier name, using escape (MemberExpression IdentifierName)
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

obj.\u0063ontinue = 42;

assert.sameValue(obj['continue'], 42, 'property exists');
