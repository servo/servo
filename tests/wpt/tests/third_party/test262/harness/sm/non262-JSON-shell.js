/*---
defines: [testJSON, testJSONSyntaxError]
---*/

function testJSON(str) {
  // Leading and trailing whitespace never affect parsing, so test the string
  // multiple times with and without whitespace around it as it's easy and can
  // potentially detect bugs.

  // Try the provided string
  try {
    JSON.parse(str);
  } catch (e) {
    throw new Test262Error("string <" + str + "> should have parsed as JSON");
  }

  // Now try the provided string with trailing whitespace
  try {
    JSON.parse(str + " ");
  } catch (e) {
    throw new Test262Error("string <" + str + " > should have parsed as JSON");
  }

  // Now try the provided string with leading whitespace
  try {
    JSON.parse(" " + str);
  } catch (e) {
    throw new Test262Error("string < " + str + "> should have parsed as JSON");
  }

  // Now try the provided string with whitespace surrounding it
  try {
    JSON.parse(" " + str + " ");
  } catch (e) {
    throw new Test262Error("string < " + str + " > should have parsed as JSON");
  }
}

function testJSONSyntaxError(str) {
  // Leading and trailing whitespace never affect parsing, so test the string
  // multiple times with and without whitespace around it as it's easy and can
  // potentially detect bugs.

  // Try the provided string
  assert.throws(SyntaxError, function() {
    JSON.parse(str);
  }, "string <" + str + "> shouldn't have parsed as JSON");

  // Now try the provided string with trailing whitespace
  assert.throws(SyntaxError, function() {
    JSON.parse(str + " ");
  }, "string <" + str + " > shouldn't have parsed as JSON");

  // Now try the provided string with leading whitespace
  assert.throws(SyntaxError, function() {
    JSON.parse(" " + str);
  }, "string < " + str + "> shouldn't have parsed as JSON");

  // Now try the provided string with whitespace surrounding it
  assert.throws(SyntaxError, function() {
    JSON.parse(" " + str + " ");
  }, "string < " + str + " > shouldn't have parsed as JSON");
}
