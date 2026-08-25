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

testJSONSyntaxError('[');
testJSONSyntaxError('[1');
testJSONSyntaxError('[1,]');
testJSONSyntaxError('[1,{');
testJSONSyntaxError('[1,}');
testJSONSyntaxError('[1,{]');
testJSONSyntaxError('[1,}]');
testJSONSyntaxError('[1,{"');
testJSONSyntaxError('[1,}"');
testJSONSyntaxError('[1,{"\\');
testJSONSyntaxError('[1,}"\\');
testJSONSyntaxError('[1,"');
testJSONSyntaxError('[1,"\\');

testJSONSyntaxError('{');
testJSONSyntaxError('{1');
testJSONSyntaxError('{,');
testJSONSyntaxError('{"');
testJSONSyntaxError('{"\\');
testJSONSyntaxError('{"\\u');
testJSONSyntaxError('{"\\uG');
testJSONSyntaxError('{"\\u0');
testJSONSyntaxError('{"\\u01');
testJSONSyntaxError('{"\\u012');
testJSONSyntaxError('{"\\u0123');
testJSONSyntaxError('{"\\u0123"');
testJSONSyntaxError('{"a"');
testJSONSyntaxError('{"a"}');
testJSONSyntaxError('{"a":');
testJSONSyntaxError('{"a",}');
testJSONSyntaxError('{"a":}');
testJSONSyntaxError('{"a":,}');
testJSONSyntaxError('{"a":5,}');
testJSONSyntaxError('{"a":5,[');
testJSONSyntaxError('{"a":5,"');
testJSONSyntaxError('{"a":5,"');
testJSONSyntaxError('{"a":5,"\\');
testJSONSyntaxError("a[false ]".substring(1, 7));

testJSONSyntaxError('this');

testJSON('[1,{}]');
testJSON('{}');
testJSON('{"a":5}');
testJSON('{"\\u0123":5}');
