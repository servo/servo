// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  String.prototype.split with regexp separator
info: bugzilla.mozilla.org/show_bug.cgi?id=614608
esid: pending
---*/
/*
 * Tests from http://xregexp.com/tests/split.html
 *
 * Copyright (C) 2007 by Steven Levithan <stevenlevithan.com>
 *
 * Distributed under the terms of the MIT license.
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:

 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.

 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 */

var ecmaSampleRe = /<(\/)?([^<>]+)>/;

var testCode = [
    ["''.split()",                   [""]],
    ["''.split(/./)",                [""]],
    ["''.split(/.?/)",               []],
    ["''.split(/.??/)",              []],
    ["'ab'.split(/a*/)",             ["", "b"]],
    ["'ab'.split(/a*?/)",            ["a", "b"]],
    ["'ab'.split(/(?:ab)/)",         ["", ""]],
    ["'ab'.split(/(?:ab)*/)",        ["", ""]],
    ["'ab'.split(/(?:ab)*?/)",       ["a", "b"]],
    ["'test'.split('')",             ["t", "e", "s", "t"]],
    ["'test'.split()",               ["test"]],
    ["'111'.split(1)",               ["", "", "", ""]],
    ["'test'.split(/(?:)/, 2)",      ["t", "e"]],
    ["'test'.split(/(?:)/, -1)",     ["t", "e", "s", "t"]],
    ["'test'.split(/(?:)/, undefined)", ["t", "e", "s", "t"]],
    ["'test'.split(/(?:)/, null)",   []],
    ["'test'.split(/(?:)/, NaN)",    []],
    ["'test'.split(/(?:)/, true)",   ["t"]],
    ["'test'.split(/(?:)/, '2')",    ["t", "e"]],
    ["'test'.split(/(?:)/, 'two')",  []],
    ["'a'.split(/-/)",               ["a"]],
    ["'a'.split(/-?/)",              ["a"]],
    ["'a'.split(/-??/)",             ["a"]],
    ["'a'.split(/a/)",               ["", ""]],
    ["'a'.split(/a?/)",              ["", ""]],
    ["'a'.split(/a??/)",             ["a"]],
    ["'ab'.split(/-/)",              ["ab"]],
    ["'ab'.split(/-?/)",             ["a", "b"]],
    ["'ab'.split(/-??/)",            ["a", "b"]],
    ["'a-b'.split(/-/)",             ["a", "b"]],
    ["'a-b'.split(/-?/)",            ["a", "b"]],
    ["'a-b'.split(/-??/)",           ["a", "-", "b"]],
    ["'a--b'.split(/-/)",            ["a", "", "b"]],
    ["'a--b'.split(/-?/)",           ["a", "", "b"]],
    ["'a--b'.split(/-??/)",          ["a", "-", "-", "b"]],
    ["''.split(/()()/)",             []],
    ["'.'.split(/()()/)",            ["."]],
    ["'.'.split(/(.?)(.?)/)",        ["", ".", "", ""]],
    ["'.'.split(/(.??)(.??)/)",      ["."]],
    ["'.'.split(/(.)?(.)?/)",        ["", ".", undefined, ""]],
    ["'A<B>bold</B>and<CODE>coded</CODE>'.split(ecmaSampleRe)",
                                     ["A", undefined, "B", "bold", "/", "B",
                                      "and", undefined, "CODE", "coded", "/",
                                      "CODE", ""]],
    ["'tesst'.split(/(s)*/)",        ["t", undefined, "e", "s", "t"]],
    ["'tesst'.split(/(s)*?/)",       ["t", undefined, "e", undefined, "s",
                                      undefined, "s", undefined, "t"]],
    ["'tesst'.split(/(s*)/)",        ["t", "", "e", "ss", "t"]],
    ["'tesst'.split(/(s*?)/)",       ["t", "", "e", "", "s", "", "s", "", "t"]],
    ["'tesst'.split(/(?:s)*/)",      ["t", "e", "t"]],
    ["'tesst'.split(/(?=s+)/)",      ["te", "s", "st"]],
    ["'test'.split('t')",            ["", "es", ""]],
    ["'test'.split('es')",           ["t", "t"]],
    ["'test'.split(/t/)",            ["", "es", ""]],
    ["'test'.split(/es/)",           ["t", "t"]],
    ["'test'.split(/(t)/)",          ["", "t", "es", "t", ""]],
    ["'test'.split(/(es)/)",         ["t", "es", "t"]],
    ["'test'.split(/(t)(e)(s)(t)/)", ["", "t", "e", "s", "t", ""]],
    ["'.'.split(/(((.((.??)))))/)",  ["", ".", ".", ".", "", "", ""]],
    ["'.'.split(/(((((.??)))))/)",   ["."]]
];

function testSplit() {
    for (var i = 0; i < testCode.length; i++) {
        var actual = eval(testCode[i][0]);
        var expected = testCode[i][1];

        assert.sameValue(actual.length, expected.length);

        for(var j=0; j<actual.length; j++) {
            assert.sameValue(actual[j], expected[j], testCode[i][0]);
        }
    }
}

testSplit();
