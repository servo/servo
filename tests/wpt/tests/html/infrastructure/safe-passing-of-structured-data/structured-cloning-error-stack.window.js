// https://tc39.es/proposal-error-stack-accessor/
// https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeinternal
//
// The [[Stack]] internal slot is now normatively serialized and deserialized
// as part of the structured clone algorithm for Error objects.

"use strict";

test(() => {
  const error = new Error("some message");
  assert_equals(typeof error.stack, "string", "original stack must be a string");

  const cloned = structuredClone(error);
  assert_equals(typeof cloned.stack, "string", "cloned stack must be a string");
  assert_equals(cloned.stack, error.stack, "cloned stack must equal original stack");
}, "structuredClone() preserves .stack on Error");

test(() => {
  const error = new TypeError("some message");
  assert_equals(typeof error.stack, "string", "original stack must be a string");

  const cloned = structuredClone(error);
  assert_equals(typeof cloned.stack, "string", "cloned stack must be a string");
  assert_equals(cloned.stack, error.stack, "cloned stack must equal original stack");
}, "structuredClone() preserves .stack on TypeError");

test(() => {
  const error = new RangeError("some message");
  assert_equals(typeof error.stack, "string", "original stack must be a string");

  const cloned = structuredClone(error);
  assert_equals(typeof cloned.stack, "string", "cloned stack must be a string");
  assert_equals(cloned.stack, error.stack, "cloned stack must equal original stack");
}, "structuredClone() preserves .stack on RangeError");

test(() => {
  const error = new EvalError("some message");
  assert_equals(typeof error.stack, "string", "original stack must be a string");

  const cloned = structuredClone(error);
  assert_equals(typeof cloned.stack, "string", "cloned stack must be a string");
  assert_equals(cloned.stack, error.stack, "cloned stack must equal original stack");
}, "structuredClone() preserves .stack on EvalError");

test(() => {
  const error = new ReferenceError("some message");
  assert_equals(typeof error.stack, "string", "original stack must be a string");

  const cloned = structuredClone(error);
  assert_equals(typeof cloned.stack, "string", "cloned stack must be a string");
  assert_equals(cloned.stack, error.stack, "cloned stack must equal original stack");
}, "structuredClone() preserves .stack on ReferenceError");

test(() => {
  const error = new SyntaxError("some message");
  assert_equals(typeof error.stack, "string", "original stack must be a string");

  const cloned = structuredClone(error);
  assert_equals(typeof cloned.stack, "string", "cloned stack must be a string");
  assert_equals(cloned.stack, error.stack, "cloned stack must equal original stack");
}, "structuredClone() preserves .stack on SyntaxError");

test(() => {
  const error = new URIError("some message");
  assert_equals(typeof error.stack, "string", "original stack must be a string");

  const cloned = structuredClone(error);
  assert_equals(typeof cloned.stack, "string", "cloned stack must be a string");
  assert_equals(cloned.stack, error.stack, "cloned stack must equal original stack");
}, "structuredClone() preserves .stack on URIError");

test(() => {
  const error = new DOMException("some message", "SyntaxError");
  assert_equals(typeof error.stack, "string", "original stack must be a string");

  const cloned = structuredClone(error);
  assert_equals(typeof cloned.stack, "string", "cloned stack must be a string");
  assert_equals(cloned.stack, error.stack, "cloned stack must equal original stack");
}, "structuredClone() preserves .stack on DOMException");

test(() => {
  let caught;
  try {
    document.createElement("");
  } catch (e) {
    caught = e;
  }
  assert_true(caught instanceof DOMException, "must be a DOMException");
  assert_equals(typeof caught.stack, "string", "original stack must be a string");

  const cloned = structuredClone(caught);
  assert_equals(typeof cloned.stack, "string", "cloned stack must be a string");
  assert_equals(cloned.stack, caught.stack, "cloned stack must equal original stack");
}, "structuredClone() preserves .stack on a thrown DOMException");

test(() => {
  const error = new Error("some message");
  const cloned = structuredClone(error);
  assert_false(cloned.hasOwnProperty("stack"),
    "cloned error must not have an own stack property");
  assert_equals(cloned.stack, error.stack,
    "cloned stack must still be accessible via the accessor");
}, "structuredClone() preserves .stack via [[Stack]] internal slot, not as own property");
