// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.3.7
description: Argument list evalution for call expresions
info: |
    A tagged template is a function call where the arguments of the call are
    derived from a TemplateLiteral. The actual arguments include a template
    object and the values produced by evaluating the expressions embedded
    within the TemplateLiteral.
---*/

var number = 5;
var string = 'str';
var object = {};
function fn() { return 'result'; }
var calls;

calls = 0;
(function() {
  return function() {
    calls++;
    assert.sameValue(
      arguments.length, 1, 'NoSubstitutionTemplate arguments length'
    );
  };
})()`NoSubstitutionTemplate`;
assert.sameValue(calls, 1, 'NoSubstitutionTemplate function invocation');

calls = 0;
(function() {
  return function(site, n, s, o, f, r) {
    calls++;
    assert.sameValue(n, number);
    assert.sameValue(s, string);
    assert.sameValue(o, object);
    assert.sameValue(f, fn);
    assert.sameValue(r, 'result');
    assert.sameValue(arguments.length, 6, 'TemplateHead arguments length');
  };
})()`TemplateHead${number}TemplateSpans${string}${object}${fn}${fn()}`;
assert.sameValue(calls, 1, 'TemplateHead function invocation');
