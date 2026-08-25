// Copyright (C) 2024 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.escape
description: Non-string inputs throw a TypeError
info: |
  RegExp.escape ( string )

  This method throws a TypeError if the input is not a string.

features: [RegExp.escape]
---*/

// Avoids a false positive when the feature is not supported
assert.sameValue(typeof RegExp.escape, 'function', 'RegExp.escape is a function');

assert.throws(TypeError, function () { RegExp.escape(123); }, 'non-string input (number) throws TypeError');
assert.throws(TypeError, function () { RegExp.escape({}); }, 'non-string input (object) throws TypeError');
assert.throws(TypeError, function () { RegExp.escape([]); }, 'non-string input (array) throws TypeError');
assert.throws(TypeError, function () { RegExp.escape(null); }, 'non-string input (null) throws TypeError');
assert.throws(TypeError, function () { RegExp.escape(undefined); }, 'non-string input (undefined) throws TypeError');
