/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [sm/non262-strict-shell.js]
description: |
  pending
esid: pending
---*/
function callFunctionBody(expr) {
  return (
    '(function f() {\n'
    + 'Object.defineProperties(arguments, {1: { writable: false },\n'
    + '                                    2: { configurable: false },\n'
    + '                                    3: { writable: false,\n'
    + '                                        configurable: false }});\n'
    + 'return (' + expr + ');\n'
    + '})(0, 1, 2, 3);');
}

assert.sameValue(testLenientAndStrict(callFunctionBody('arguments[0] = 42'),
                              returns(42), returns(42)),
         true);

assert.sameValue(testLenientAndStrict(callFunctionBody('delete arguments[0]'),
                              returns(true), returns(true)),
         true);


assert.sameValue(testLenientAndStrict(callFunctionBody('arguments[1] = 42'),
                              returns(42), raisesException(TypeError)),
         true);

assert.sameValue(testLenientAndStrict(callFunctionBody('delete arguments[1]'),
                              returns(true), returns(true)),
         true);


assert.sameValue(testLenientAndStrict(callFunctionBody('arguments[2] = 42'),
                              returns(42), returns(42)),
         true);

assert.sameValue(testLenientAndStrict(callFunctionBody('delete arguments[2]'),
                              returns(false), raisesException(TypeError)),
         true);


assert.sameValue(testLenientAndStrict(callFunctionBody('arguments[3] = 42'),
                              returns(42), raisesException(TypeError)),
         true);

assert.sameValue(testLenientAndStrict(callFunctionBody('delete arguments[3]'),
                              returns(false), raisesException(TypeError)),
         true);


