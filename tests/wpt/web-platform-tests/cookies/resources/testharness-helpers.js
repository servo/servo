// Given an array of potentially asynchronous tests, this function will execute
// each in serial, ensuring that one and only one test is executing at a time.
//
// The test array should look like this:
//
//
//     var tests = [
//       [
//         "Test description goes here.",
//         function () {
//           // Test code goes here. `this` is bound to the test object.
//         }
//       ],
//       ...
//     ];
//
// The |setup| and |teardown| arguments are functions which are executed before
// and after each test, respectively.
function executeTestsSerially(testList, setup, teardown) {
  var tests = testList.map(function (t) {
    return {
      test: async_test(t[0]),
      code: t[1]
    };
  });

  var executeNextTest = function () {
    var current = tests.shift();
    if (current === undefined) {
      return;
    }

    // Setup the test fixtures.
    if (setup) {
      setup();
    }

    // Bind a callback to tear down the test fixtures.
    if (teardown) {
      current.test.add_cleanup(teardown);
    }

    // Execute the test.
    current.test.step(current.code);
  };

  add_result_callback(function () { setTimeout(executeNextTest, 0) });
  executeNextTest();
}
