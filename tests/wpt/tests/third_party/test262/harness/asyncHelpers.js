// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    A collection of assertion and wrapper functions for testing asynchronous built-ins.
defines: [asyncTest, assert.throwsAsync]
---*/

/**
 * Defines the **sole** asynchronous test of a file.
 * @see {@link ../docs/rfcs/async-helpers.md} for background.
 *
 * @param {Function} testFunc a callback whose returned promise indicates test results
 *   (fulfillment for success, rejection for failure)
 * @returns {void}
 */
function asyncTest(testFunc) {
  if (!Object.prototype.hasOwnProperty.call(globalThis, "$DONE")) {
    throw new Test262Error("asyncTest called without async flag");
  }
  if (typeof testFunc !== "function") {
    $DONE(new Test262Error("asyncTest called with non-function argument"));
    return;
  }
  try {
    testFunc().then(
      function () {
        $DONE();
      },
      function (error) {
        $DONE(error);
      }
    );
  } catch (syncError) {
    $DONE(syncError);
  }
}

/**
 * Asserts that a callback asynchronously throws an instance of a particular
 * error (i.e., returns a promise whose rejection value is an object referencing
 * the constructor).
 *
 * @param {Function} expectedErrorConstructor the expected constructor of the
 *   rejection value
 * @param {Function} func the callback
 * @param {string} [message] the prefix to use for failure messages
 * @returns {Promise<void>} fulfills if the expected error is thrown,
 *   otherwise rejects
 */
assert.throwsAsync = function (expectedErrorConstructor, func, message) {
  return new Promise(function (resolve) {
    var fail = function (detail) {
      if (message === undefined) {
        throw new Test262Error(detail);
      }
      throw new Test262Error(message + " " + detail);
    };
    if (typeof expectedErrorConstructor !== "function") {
      fail("assert.throwsAsync called with an argument that is not an error constructor");
    }
    if (typeof func !== "function") {
      fail("assert.throwsAsync called with an argument that is not a function");
    }
    var expectedName = expectedErrorConstructor.name;
    var expectation = "Expected a " + expectedName + " to be thrown asynchronously";
    var res;
    try {
      res = func();
    } catch (thrown) {
      fail(expectation + " but the function threw synchronously");
    }
    if (res === null || typeof res !== "object" || typeof res.then !== "function") {
      fail(expectation + " but result was not a thenable");
    }
    var onResFulfilled, onResRejected;
    var resSettlementP = new Promise(function (onFulfilled, onRejected) {
      onResFulfilled = onFulfilled;
      onResRejected = onRejected;
    });
    try {
      res.then(onResFulfilled, onResRejected)
    } catch (thrown) {
      fail(expectation + " but .then threw synchronously");
    }
    resolve(resSettlementP.then(
      function () {
        fail(expectation + " but no exception was thrown at all");
      },
      function (thrown) {
        var actualName;
        if (thrown === null || typeof thrown !== "object") {
          fail(expectation + " but thrown value was not an object");
        } else if (thrown.constructor !== expectedErrorConstructor) {
          actualName = thrown.constructor.name;
          if (expectedName === actualName) {
            fail(expectation +
              " but got a different error constructor with the same name");
          }
          fail(expectation + " but got a " + actualName);
        }
      }
    ));
  });
};
