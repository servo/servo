// Ends a manual test. Must be called before any async tests are started.
function skipManualTest() {
    test(function() { assert_true(false); }, "Manual Test Skipped");
    done();
}

var stepInstructions = [];
var testNames = [];
var stepFunctions = [];
var steps;
var curStep = 0;

// Adds a manual test step to the test. A test will add a series of steps,
// along with instructions.  Once all the tests steps are added, the test can
// be run by continually running the nextStep() function. All manual test steps
// must be added before calling nextStep.
//
// |func| A function to be executed at the given step. This function can include
//        testharness assertions if |testName| is provided. If this is the last
//        step, the |done()| function (used for manual testharness.js tests)
//        will be called after |func| is executed.
// |testName| If provided, the |func| will be wrapped in a testharness.js
//            async_test with this name. If null, |func| will be executed as a
//            free function.
// |instructions| The text to display to the user. Note, these are shown after
//                step is executed so these should be instructions to setup the
//                checks in the next step.
function addManualTestStep(func, testName, instructions) {
    stepFunctions.push(func);
    testNames.push(testName);
    stepInstructions.push(instructions);
}

// Runs the next step of the test. This must be called only after all test steps
// have been added using |addManualTestStep|.
//
// |callbackFunc| If provided, will be called with a single argument being the
//                instruction string for the current step. Use this to update
//                any necessary UI.
function nextStep(callbackFunc) {
    if (curStep == 0)
      _startManualTest();

    if (typeof(callbackFunc) === 'function')
        callbackFunc(stepInstructions[curStep]);

    steps[curStep]();
    curStep++;
}

function _startManualTest() {
    steps = [];
    for (let i = 0; i < stepFunctions.length; ++i) {
        var stepFunc = stepFunctions[i];
        var testName = testNames[i];
        if (testName) {
            steps.push(async_test(testName).step_func(function() {
                stepFunctions[i]();
                this.done();
                if (i == stepFunctions.length - 1)
                    done();
            }));
        } else {
            steps.push(function() {
                stepFunctions[i]();
                if (i == stepFunctions.length - 1)
                    done();
            });
        }
    }
}
