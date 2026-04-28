/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [sm/non262-JSON-shell.js]
description: |
  'JSON.parse should reject {"a" : "b",} or [1,]
esid: pending
---*/

testJSON('[]');
testJSON('[1]');
testJSON('["a"]');
testJSON('{}');
testJSON('{"a":1}');
testJSON('{"a":"b"}');
testJSON('{"a":true}');
testJSON('[{}]');

testJSONSyntaxError('[1,]');
testJSONSyntaxError('["a",]');
testJSONSyntaxError('{,}');
testJSONSyntaxError('{"a":1,}');
testJSONSyntaxError('{"a":"b",}');
testJSONSyntaxError('{"a":true,}');
testJSONSyntaxError('[{,}]');
testJSONSyntaxError('[[1,]]');
testJSONSyntaxError('[{"a":"b",}]');
