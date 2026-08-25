// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Test replace function with duplicate names in alteration.
features: [regexp-duplicate-named-groups]
---*/

assert.sameValue(
    '2xyy', 'xxyy'.replace(/(?:(?:(?<a>x)|(?<a>y))\k<a>)/, '2$<a>'));
assert.sameValue(
    'x2zyyxxy',
    'xzzyyxxy'.replace(
        /(?:(?:(?<a>x)|(?<a>y)|(a)|(?<b>b)|(?<a>z))\k<a>)/, '2$<a>'));
assert.sameValue(
    '2x(x,)yy', 'xxyy'.replace(/(?:(?:(?<a>x)|(?<a>y))\k<a>)/, '2$<a>($1,$2)'));
assert.sameValue(
    'x2z(,,,,z)yyxxy',
    'xzzyyxxy'.replace(
        /(?:(?:(?<a>x)|(?<a>y)|(a)|(?<b>b)|(?<a>z))\k<a>)/,
        '2$<a>($1,$2,$3,$4,$5)'));
assert.sameValue(
    '2x2y', 'xxyy'.replace(/(?:(?:(?<a>x)|(?<a>y))\k<a>)/g, '2$<a>'));
assert.sameValue(
    'x2z2y2xy',
    'xzzyyxxy'.replace(
        /(?:(?:(?<a>x)|(?<a>y)|(a)|(?<b>b)|(?<a>z))\k<a>)/g, '2$<a>'));
