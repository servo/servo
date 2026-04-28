// Copyright (C) 2016 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Ensure that the regular expression generally distinguishes between valid
    and invalid forms of the NativeFunction grammar production.
includes: [nativeFunctionMatcher.js]
---*/

[
  'function(){[native code]}',
  'function(){ [native code] }',
  'function ( ) { [ native code ] }',
  'function a(){ [native code] }',
  'function a(){ /* } */ [native code] }',
  `function a() {
    // test
    [native code]
    /* test */
  }`,
  'function(a, b = function() { []; }) { [native code] }',
  'function [Symbol.xyz]() { [native code] }',
  'function [x[y][z[d]()]]() { [native code] }',
  'function ["]"] () { [native code] }',
  'function [\']\'] () { [native code] }',
  '/* test */ function() { [native code] }',
  'function() { [native code] } /* test */',
  'function() { [native code] } // test',
].forEach((s) => {
  try {
    validateNativeFunctionSource(s);
  } catch (unused) {
    throw new Error(`${JSON.stringify(s)} should pass`);
  }
});

[
  'native code',
  'function() {}',
  'function(){ "native code" }',
  'function(){ [] native code }',
  'function()) { [native code] }',
  'function(() { [native code] }',
  'function []] () { [native code] }',
  'function [[] () { [native code] }',
  'function ["]] () { [native code] }',
  'function [\']] () { [native code] }',
  'function() { [native code] /* }',
  '// function() { [native code] }',
].forEach((s) => {
  let fail = false;
  try {
    validateNativeFunctionSource(s);
    fail = true;
  } catch (unused) {}
  if (fail) {
    throw new Error(`${JSON.stringify(s)} should fail`);
  }
});
