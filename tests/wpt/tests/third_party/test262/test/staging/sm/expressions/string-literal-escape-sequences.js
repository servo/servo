/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  \\u and \\x must be followed by the appropriate number of hex digits or else it is a syntax error
info: bugzilla.mozilla.org/show_bug.cgi?id=663300
esid: pending
---*/

function expectSyntaxError(str)
{
  assert.throws(SyntaxError, function() {
    eval(str);
  }, "syntax error evaluating " + str);
}

expectSyntaxError('"\\x"');
expectSyntaxError('"\\x0"');
expectSyntaxError('"\\x1"');
expectSyntaxError('"\\x2"');
expectSyntaxError('"\\x3"');
expectSyntaxError('"\\x4"');
expectSyntaxError('"\\x5"');
expectSyntaxError('"\\x6"');
expectSyntaxError('"\\x7"');
expectSyntaxError('"\\x8"');
expectSyntaxError('"\\x9"');
expectSyntaxError('"\\xA"');
expectSyntaxError('"\\xB"');
expectSyntaxError('"\\xC"');
expectSyntaxError('"\\xD"');
expectSyntaxError('"\\xE"');
expectSyntaxError('"\\xF"');
expectSyntaxError('"\\xG"');
expectSyntaxError('"\\x0G"');
expectSyntaxError('"\\x1G"');
expectSyntaxError('"\\x2G"');
expectSyntaxError('"\\x3G"');
expectSyntaxError('"\\x4G"');
expectSyntaxError('"\\x5G"');
expectSyntaxError('"\\x6G"');
expectSyntaxError('"\\x7G"');
expectSyntaxError('"\\x8G"');
expectSyntaxError('"\\x9G"');
expectSyntaxError('"\\xAG"');
expectSyntaxError('"\\xBG"');
expectSyntaxError('"\\xCG"');
expectSyntaxError('"\\xDG"');
expectSyntaxError('"\\xEG"');
expectSyntaxError('"\\xFG"');
expectSyntaxError('"\\xGG"');

expectSyntaxError('"\\u"');
expectSyntaxError('"\\u0"');
expectSyntaxError('"\\u1"');
expectSyntaxError('"\\u2"');
expectSyntaxError('"\\u3"');
expectSyntaxError('"\\u4"');
expectSyntaxError('"\\u5"');
expectSyntaxError('"\\u6"');
expectSyntaxError('"\\u7"');
expectSyntaxError('"\\u8"');
expectSyntaxError('"\\u9"');
expectSyntaxError('"\\uA"');
expectSyntaxError('"\\uB"');
expectSyntaxError('"\\uC"');
expectSyntaxError('"\\uD"');
expectSyntaxError('"\\uE"');
expectSyntaxError('"\\uF"');
expectSyntaxError('"\\uG"');
expectSyntaxError('"\\u00"');
expectSyntaxError('"\\u11"');
expectSyntaxError('"\\u22"');
expectSyntaxError('"\\u33"');
expectSyntaxError('"\\u44"');
expectSyntaxError('"\\u55"');
expectSyntaxError('"\\u66"');
expectSyntaxError('"\\u77"');
expectSyntaxError('"\\u88"');
expectSyntaxError('"\\u99"');
expectSyntaxError('"\\uAA"');
expectSyntaxError('"\\uBB"');
expectSyntaxError('"\\uCC"');
expectSyntaxError('"\\uDD"');
expectSyntaxError('"\\uEE"');
expectSyntaxError('"\\uFF"');
expectSyntaxError('"\\uGG"');
expectSyntaxError('"\\u000"');
expectSyntaxError('"\\u111"');
expectSyntaxError('"\\u222"');
expectSyntaxError('"\\u333"');
expectSyntaxError('"\\u444"');
expectSyntaxError('"\\u555"');
expectSyntaxError('"\\u666"');
expectSyntaxError('"\\u777"');
expectSyntaxError('"\\u888"');
expectSyntaxError('"\\u999"');
expectSyntaxError('"\\uAAA"');
expectSyntaxError('"\\uBBB"');
expectSyntaxError('"\\uCCC"');
expectSyntaxError('"\\uDDD"');
expectSyntaxError('"\\uEEE"');
expectSyntaxError('"\\uFFF"');
expectSyntaxError('"\\uGGG"');
expectSyntaxError('"\\u000G"');
expectSyntaxError('"\\u111G"');
expectSyntaxError('"\\u222G"');
expectSyntaxError('"\\u333G"');
expectSyntaxError('"\\u444G"');
expectSyntaxError('"\\u555G"');
expectSyntaxError('"\\u666G"');
expectSyntaxError('"\\u777G"');
expectSyntaxError('"\\u888G"');
expectSyntaxError('"\\u999G"');
expectSyntaxError('"\\uAAAG"');
expectSyntaxError('"\\uBBBG"');
expectSyntaxError('"\\uCCCG"');
expectSyntaxError('"\\uDDDG"');
expectSyntaxError('"\\uEEEG"');
expectSyntaxError('"\\uFFFG"');
expectSyntaxError('"\\uGGGG"');

assert.sameValue(eval('"a\\\rb"'), "ab");
assert.sameValue(eval('"a\\\nb"'), "ab");
assert.sameValue(eval('"a\\\r\nb"'), "ab");
assert.sameValue(eval('"a\\\u2028b"'), "ab");
assert.sameValue(eval('"a\\\u2029b"'), "ab");
