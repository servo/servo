// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    ImportCall is a CallExpression and can be used before a template literal
esid: prod-ImportCall
info: |
  CallExpression:
    ImportCall
    CallExpression TemplateLiteral
    CallExpression Arguments
features: [dynamic-import]
---*/

// valid syntax, but fails on runtime evaluation

assert.throws(TypeError, () => {
    import('./empty_FIXTURE.js')``;
});

assert.throws(TypeError, () => {
    import('./empty_FIXTURE.js')`something`;
});

assert.throws(TypeError, () => {
    import('./empty_FIXTURE.js')`${42}`;
});
