// This file was procedurally generated from the following sources:
// - src/identifier-names/default-escaped-ext.case
// - src/identifier-names/default/member-expr.template
/*---
description: default is a valid identifier name, using extended escape (MemberExpression IdentifierName)
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

obj.def\u{61}ult = 42;

assert.sameValue(obj['default'], 42, 'property exists');
