// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Parentheses of the form ( Disjunction ) serve both to group the components of the Disjunction pattern together and to save the result of the match.
    The result can be used either in a backreference (\ followed by a nonzero decimal number),
    referenced in a replace string,
    or returned as part of an array from the regular expression matching function
es5id: 15.10.2.8_A3_T18
description: "see bug  http:bugzilla.mozilla.org/show_bug.cgi?id=169534"
---*/

var __replaced = "To sign up click |here|https:www.xxxx.org/subscribe.htm|".replace(/(\|)([\w\x81-\xff ]*)(\|)([\/a-z][\w:\/\.]*\.[a-z]{3,4})(\|)/ig, '<a href="$4">$2</a>');

var __expected = 'To sign up click <a href="https:www.xxxx.org/subscribe.htm">here</a>';

assert.sameValue(__replaced, __expected, 'The value of __replaced is expected to equal the value of __expected');
