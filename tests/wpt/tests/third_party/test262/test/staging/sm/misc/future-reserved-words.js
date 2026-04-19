/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  Implement FutureReservedWords per-spec
info: bugzilla.mozilla.org/show_bug.cgi?id=497869
esid: pending
---*/

var futureReservedWords =
  [
   "class",
   "const",
   "enum",
   "export",
   "extends",
   "import",
   "super",
  ];

var strictFutureReservedWords =
  [
   "implements",
   "interface",
   "let",
   "package",
   "private",
   "protected",
   "public",
   "static",
   "yield",
  ];

function testNormalAndStrict(word, code, message) {
  if (strictFutureReservedWords.includes(word)) {
    eval(code);
  } else {
    assert(futureReservedWords.includes(word));
    assert.throws(SyntaxError, function() {
      eval(code);
    }, word + ": normal " + message);
  }

  assert.throws(SyntaxError, function() {
    eval("'use strict'; " + code);
  }, word + ": strict " + message);
}

function testWord(word) {
  // USE AS LHS FOR ASSIGNMENT
  testNormalAndStrict(word, word + " = 'foo';", "assignment");

  // USE AS DESTRUCTURING SHORTHAND
  testNormalAndStrict(word, "({ " + word + " } = 'foo');", "destructuring shorthand");

  // USE IN VARIABLE DECLARATION
  testNormalAndStrict(word, "var " + word + ";", "var");

  // USE IN FOR-IN VARIABLE DECLARATION
  testNormalAndStrict(word, "for (var " + word + " in {});", "for-in var");

  // USE AS CATCH IDENTIFIER
  testNormalAndStrict(word, "try { } catch (" + word + ") { }", "catch var");

  // USE AS LABEL
  testNormalAndStrict(word, word + ": while (false);", "label");

  // USE AS ARGUMENT NAME IN FUNCTION DECLARATION
  testNormalAndStrict(word, "function foo(" + word + ") { }", "function argument");

  assert.throws(SyntaxError, function() {
    eval("function foo(" + word + ") { 'use strict'; }");
  }, word + ": function argument retroactively strict");

  // USE AS ARGUMENT NAME IN FUNCTION EXPRESSION
  testNormalAndStrict(word, "var s = (function foo(" + word + ") { });", "function expression argument");

  assert.throws(SyntaxError, function() {
    eval("var s = (function foo(" + word + ") { 'use strict'; });");
  }, word + ": function expression argument retroactively strict");

  // USE AS ARGUMENT NAME WITH FUNCTION CONSTRUCTOR
  if (strictFutureReservedWords.includes(word)) {
    Function(word, "return 17");
  } else {
    assert.throws(SyntaxError, function() {
      Function(word, "return 17");
    }, word + ": argument with normal Function");
  }

  assert.throws(SyntaxError, function() {
    Function(word, "'use strict'; return 17");
  }, word + ": argument with strict Function");

  // USE AS ARGUMENT NAME IN PROPERTY SETTER
  testNormalAndStrict(word, "var o = { set x(" + word + ") { } };", "property setter argument");

  assert.throws(SyntaxError, function() {
    eval("var o = { set x(" + word + ") { 'use strict'; } };");
  }, word + ": property setter argument retroactively strict");

  // USE AS FUNCTION NAME IN FUNCTION DECLARATION
  testNormalAndStrict(word, "function " + word + "() { }", "function name");

  assert.throws(SyntaxError, function() {
    eval("function " + word + "() { 'use strict'; }");
  }, word + ": function name retroactively strict");

  // USE AS FUNCTION NAME IN FUNCTION EXPRESSION
  testNormalAndStrict(word, "var s = (function " + word + "() { });", "function expression name");

  assert.throws(SyntaxError, function() {
    eval("var s = (function " + word + "() { 'use strict'; });");
  }, word + ": function expression name retroactively strict");
}

futureReservedWords.forEach(testWord);
strictFutureReservedWords.forEach(testWord);
