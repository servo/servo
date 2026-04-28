// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    ImportCall is a CallExpression and Expression, so it can be wrapped
    for new expressions, while the same production is not possible without
    the parentheses wrapping.
esid: prod-ImportCall
info: |
  CallExpression:
    ImportCall

  ImportCall :
    import( AssignmentExpression[+In, ?Yield] )

  NewExpression :
    MemberExpression
    new NewExpression

  MemberExpression :
    PrimaryExpression

  PrimaryExpression :
    CoverParenthesizedExpressionAndArrowParameterList
features: [dynamic-import]
---*/

assert.throws(TypeError, () => {
    new (import(''))
});

assert.throws(TypeError, () => {
    new (function() {}, import(''))
});

assert.sameValue(
    typeof new (import(''), function() {}),
    'object',
);
