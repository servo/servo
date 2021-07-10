// If scrollbars affect layout (i.e. what the CSS Overflow spec calls "classic
// scrollbars", as opposed to overlay scrollbars), return the scrollbar
// thickness in CSS pixels. Returns 0 otherwise.
function calculateScrollbarThickness() {
    var container = document.createElement("div");
    container.style.width = "100px";
    container.style.height = "100px";
    container.style.position = "absolute";
    container.style.visibility = "hidden";
    container.style.overflow = "auto";

    document.body.appendChild(container);

    var widthBefore = container.clientWidth;
    var longContent = document.createElement("div");
    longContent.style.height = "1000px";
    container.appendChild(longContent);

    var widthAfter = container.clientWidth;

    container.remove();

    return widthBefore - widthAfter;
}

// Puts up a widget on screen instructing the user to pinch-zoom in to the
// given scale. The widget is sized such that the given scale is achieved. The
// widget is placed at (x, y) on the page. A button on the widget is used by
// the user to let the widget know that the user has finished. If a callback is
// provided, it will be called when the user dismisses the widget.
function showPinchWidget(scale, x, y, callback) {
    var border = 10;
    var width = window.innerWidth / scale - border;
    var height = window.innerHeight / scale - border;

    var box = document.createElement("div");
    box.style.width = width + "px";
    box.style.height = height + "px";

    // Adjust the x/y coordinates by the border width. We want the box to
    // appear in a place such that if the user gets the window edges exactly on
    // the half-point of the border they end up at x/y
    box.style.left = x - border/2 + "px";
    box.style.top = y - border/2 + "px";

    box.style.position = "absolute";
    box.style.backgroundColor = "coral";
    box.style.border = border + "px solid blue";
    box.style.borderBottom = "0";
    box.style.overflow = "auto";

    var oldDocumentOverflow = document.documentElement.style.overflow;

    var instructions = document.createElement("p");
    instructions.innerText =
        "Pinch-zoom and align this box so that the left, right, and top " +
        "window edges are over the border on each side. When done, click the " +
        "'DONE' button above";
    instructions.style.textAlign = "center";
    instructions.style.fontSize = "medium";

    var button = document.createElement("button");
    button.innerText = "DONE";
    button.style.width = "50%";
    button.style.height = "20%";
    button.style.fontSize = "medium";
    button.style.marginLeft = "25%";
    button.addEventListener("click", function() {
        box.remove();
        document.documentElement.style.overflow = oldDocumentOverflow;
        if (callback)
            callback();
    });

    box.appendChild(button);
    box.appendChild(instructions);

    document.documentElement.style.overflow = "hidden";

    document.body.appendChild(box);
}

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
