// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Parentheses of the form ( Disjunction ) serve both to group the components of the Disjunction pattern together and to save the result of the match.
    The result can be used either in a backreference (\ followed by a nonzero decimal number),
    referenced in a replace string,
    or returned as part of an array from the regular expression matching function
es5id: 15.10.2.8_A3_T17
description: "see bug http:bugzilla.mozilla.org/show_bug.cgi?id=169497"
---*/

var __body="";
__body += '<body onXXX="alert(event.type);">\n';
__body += '<p>Kibology for all<\/p>\n';
__body += '<p>All for Kibology<\/p>\n';
__body += '<\/body>';

var __html="";
__html += '<html>\n';
__html += __body;
__html += '\n<\/html>';

var __executed = /<body.*>((.*\n?)*?)<\/body>/i.exec(__html);

var __expected = [__body, '\n<p>Kibology for all</p>\n<p>All for Kibology</p>\n', '<p>All for Kibology</p>\n'];
__expected.index = 7;
__expected.input = __html;

assert.sameValue(
  __executed.length,
  __expected.length,
  'The value of __executed.length is expected to equal the value of __expected.length'
);

assert.sameValue(
  __executed.index,
  __expected.index,
  'The value of __executed.index is expected to equal the value of __expected.index'
);

assert.sameValue(
  __executed.input,
  __expected.input,
  'The value of __executed.input is expected to equal the value of __expected.input'
);

for(var index=0; index<__expected.length; index++) {
  assert.sameValue(
    __executed[index],
    __expected[index],
    'The value of __executed[index] is expected to equal the value of __expected[index]'
  );
}
