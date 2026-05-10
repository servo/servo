// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    RegExp.prototype.exec behavior depends on global property.
    If global is true and lastIndex not changed manually,
    next exec calling start to match from position where current match finished
es5id: 15.10.6.2_A3_T6
description: RegExp is /(\d+)/g and tested string is "123 456 789"
---*/

var __re = /(\d+)/g;

var __matched = [];

var __expected = ["123","456","789"];

do{
    var __executed = __re.exec("123 456 789");
    if (__executed !== null) {
        __matched.push(__executed[0]);
    } else {
        break;
    }
}while(true);

assert.sameValue(
  __expected.length,
  __matched.length,
  'The value of __expected.length is expected to equal the value of __matched.length'
);

for(var index=0; index<__expected.length; index++) {
  assert.sameValue(
    __expected[index],
    __matched[index],
    'The value of __expected[index] is expected to equal the value of __matched[index]'
  );
}
