// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    ImportCall is a CallExpression and can be used before arguments
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
    import('./empty_FIXTURE.js')();
}, 'empty arguments');

assert.throws(TypeError, () => {
    import('./empty_FIXTURE.js')(1,);
}, 'arguments with trailing comma');

assert.throws(TypeError, () => {
    import('./empty_FIXTURE.js')(1, 2);
}, '2 arguments');

assert.throws(TypeError, () => {
    import('./empty_FIXTURE.js')(...[]);
}, 'spread args');
