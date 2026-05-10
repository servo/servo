/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [sm/non262-JSON-shell.js]
description: |
  pending
esid: pending
---*/

testJSONSyntaxError('-');
testJSONSyntaxError('+');
testJSONSyntaxError('-f');
testJSONSyntaxError('+f');
testJSONSyntaxError('00');
testJSONSyntaxError('01');
testJSONSyntaxError('1.');
testJSONSyntaxError('1.0e');
testJSONSyntaxError('1.0e+');
testJSONSyntaxError('1.0e-');
testJSONSyntaxError('1.0e+z');
testJSONSyntaxError('1.0e-z');
testJSONSyntaxError('1.0ee');
testJSONSyntaxError('1.e1');
testJSONSyntaxError('1.e+1');
testJSONSyntaxError('1.e-1');
testJSONSyntaxError('.');
testJSONSyntaxError('.1');
testJSONSyntaxError('.1e');
testJSONSyntaxError('.1e1');
testJSONSyntaxError('.1e+1');
testJSONSyntaxError('.1e-1');
