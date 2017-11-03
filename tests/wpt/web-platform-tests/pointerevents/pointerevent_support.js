var All_Pointer_Events = [
        "pointerdown",
        "pointerup",
        "pointercancel",
        "pointermove",
        "pointerover",
        "pointerout",
        "pointerenter",
        "pointerleave",
        "gotpointercapture",
        "lostpointercapture"];

// Check for conformance to PointerEvent interface
// TA: 1.1, 1.2, 1.6, 1.7, 1.8, 1.9, 1.10, 1.11, 1.12, 1.13
function check_PointerEvent(event, testNamePrefix) {
    if (testNamePrefix === undefined)
        testNamePrefix = "";

    // Use expectedPointerType if set otherwise just use the incoming event pointerType in the test name.
    var pointerTestName = testNamePrefix + ' ' + (expectedPointerType == null ? event.pointerType : expectedPointerType) + ' ' + event.type;

    if (expectedPointerType != null) {
        test(function () {
            assert_equals(event.pointerType, expectedPointerType, "pointerType should be the one specified in the test page.");
        }, pointerTestName + " event pointerType is correct.");
    }

    test(function () {
        assert_true(event instanceof event.target.ownerDocument.defaultView.PointerEvent, "event is a PointerEvent event");
    }, pointerTestName + " event is a PointerEvent event");


    // Check attributes for conformance to WebIDL:
    // * attribute exists
    // * has proper type
    // * if the attribute is "readonly", it cannot be changed
    // TA: 1.1, 1.2
    var idl_type_check = {
        "long": function (v) { return typeof v === "number" && Math.round(v) === v; },
        "float": function (v) { return typeof v === "number"; },
        "string": function (v) { return typeof v === "string"; },
        "boolean": function (v) { return typeof v === "boolean" }
    };
    [
        ["readonly", "long", "pointerId"],
        ["readonly", "float", "width"],
        ["readonly", "float", "height"],
        ["readonly", "float", "pressure"],
        ["readonly", "long", "tiltX"],
        ["readonly", "long", "tiltY"],
        ["readonly", "string", "pointerType"],
        ["readonly", "boolean", "isPrimary"],
        ["readonly", "long", "detail", 0]
    ].forEach(function (attr) {
        var readonly = attr[0];
        var type = attr[1];
        var name = attr[2];
        var value = attr[3];

        // existence check
        test(function () {
            assert_true(name in event, name + " attribute in " + event.type + " event");
        }, pointerTestName + "." + name + " attribute exists");

        // readonly check
        if (readonly === "readonly") {
            test(function () {
                assert_readonly(event.type, name, event.type + "." + name + " cannot be changed");
            }, pointerTestName + "." + name + " is readonly");
        }

        // type check
        test(function () {
            assert_true(idl_type_check[type](event[name]), name + " attribute of type " + type);
        }, pointerTestName + "." + name + " IDL type " + type + " (JS type was " + typeof event[name] + ")");

        // value check if defined
        if (value != undefined) {
            test(function () {
                assert_equals(event[name], value, name + " attribute value");
            }, pointerTestName + "." + name + " value is " + value + ".");
        }
    });


    // Check the pressure value
    // TA: 1.6, 1.7, 1.8
    test(function () {
        // TA: 1.6
        assert_greater_than_equal(event.pressure, 0, "pressure is greater than or equal to 0");
        assert_less_than_equal(event.pressure, 1, "pressure is less than or equal to 1");

        if (event.buttons === 0) {
            assert_equals(event.pressure, 0, "pressure is 0 for mouse with no buttons pressed");
        }

        // TA: 1.7, 1.8
        if (event.pointerType === "mouse") {
            if (event.buttons !== 0) {
                assert_equals(event.pressure, 0.5, "pressure is 0.5 for mouse with a button pressed");
            }
        }
    }, pointerTestName + ".pressure value is valid");

    // Check mouse-specific properties
    if (event.pointerType === "mouse") {
        // TA: 1.9, 1.10, 1.13
        test(function () {
            assert_equals(event.width, 1, "width of mouse should be 1");
            assert_equals(event.height, 1, "height of mouse should be 1");
            assert_equals(event.tiltX, 0, event.type + ".tiltX is 0 for mouse");
            assert_equals(event.tiltY, 0, event.type + ".tiltY is 0 for mouse");
            assert_true(event.isPrimary, event.type + ".isPrimary is true for mouse");
        }, pointerTestName + " properties for pointerType = mouse");
        // Check properties for pointers other than mouse
    }
}

function showPointerTypes() {
    var complete_notice = document.getElementById("complete-notice");
    var pointertype_log = document.getElementById("pointertype-log");
    var pointertypes = Object.keys(detected_pointertypes);
    pointertype_log.innerHTML = pointertypes.length ?
        pointertypes.join(",") : "(none)";
    complete_notice.style.display = "block";
}

function showLoggedEvents() {
    var event_log_elem = document.getElementById("event-log");
    event_log_elem.innerHTML = event_log.length ? event_log.join(", ") : "(none)";

    var complete_notice = document.getElementById("complete-notice");
    complete_notice.style.display = "block";
}

function log(msg, el) {
    if (++count > 10){
      count = 0;
      el.innerHTML = ' ';
    }
    el.innerHTML = msg + '; ' + el.innerHTML;
}

 function failOnScroll() {
    assert_true(false,
    "scroll received while shouldn't");
}

function updateDescriptionNextStep() {
    document.getElementById('desc').innerHTML = "Test Description: Try to scroll text RIGHT.";
}

function updateDescriptionComplete() {
    document.getElementById('desc').innerHTML = "Test Description: Test complete";
}

function updateDescriptionSecondStepTouchActionElement(target, scrollReturnInterval) {
    window.step_timeout(function() {
    objectScroller(target, 'up', 0);}
    , scrollReturnInterval);
    document.getElementById('desc').innerHTML = "Test Description: Try to scroll element RIGHT moving your outside of the red border";
}

function updateDescriptionThirdStepTouchActionElement(target, scrollReturnInterval) {
    window.step_timeout(function() {
    objectScroller(target, 'left', 0);}
    , scrollReturnInterval);
    document.getElementById('desc').innerHTML = "Test Description: Try to scroll element DOWN then RIGHT starting your touch inside of the element. Then tap complete button";
}

function updateDescriptionFourthStepTouchActionElement(target, scrollReturnInterval) {
    document.getElementById('desc').innerHTML = "Test Description: Try to scroll element RIGHT starting your touch inside of the element";
}

function objectScroller(target, direction, value) {
    if (direction == 'up') {
        target.scrollTop = 0;
    } else if (direction == 'left') {
        target.scrollLeft = 0;
    }
}

function sPointerCapture(e) {
    try {
        target0.setPointerCapture(e.pointerId);
    }
    catch(e) {
    }
}

function rPointerCapture(e) {
    try {
        captureButton.value = 'Set Capture';
        target0.releasePointerCapture(e.pointerId);
    }
    catch(e) {
    }
}

var globalPointerEventTest = null;
var expectedPointerType = null;
const ALL_POINTERS = ['mouse', 'touch', 'pen'];
const HOVERABLE_POINTERS = ['mouse', 'pen'];
const NOHOVER_POINTERS = ['touch'];

function MultiPointerTypeTest(testName, types) {
    this.testName = testName;
    this.types = types;
    this.currentTypeIndex = 0;
    this.currentTest = null;
    this.createNextTest();
}

MultiPointerTypeTest.prototype.step = function(op) {
    this.currentTest.step(op);
}

MultiPointerTypeTest.prototype.skip = function() {
    var prevTest = this.currentTest;
    this.createNextTest();
    prevTest.timeout();
}

MultiPointerTypeTest.prototype.done = function() {
    if (this.currentTest.status != 1) {
        var prevTest = this.currentTest;
        this.createNextTest();
        if (prevTest != null)
            prevTest.done();
    }
}

MultiPointerTypeTest.prototype.step = function(stepFunction) {
    this.currentTest.step(stepFunction);
}

MultiPointerTypeTest.prototype.createNextTest = function() {
    if (this.currentTypeIndex < this.types.length) {
        var pointerTypeDescription = document.getElementById('pointerTypeDescription');
        document.getElementById('pointerTypeDescription').innerHTML = "Follow the test instructions with <span style='color: red'>" + this.types[this.currentTypeIndex] + "</span>. If you don't have the device <a href='javascript:;' onclick='globalPointerEventTest.skip()'>skip it</a>.";
        this.currentTest = async_test(this.types[this.currentTypeIndex] + ' ' + this.testName);
        expectedPointerType = this.types[this.currentTypeIndex];
        this.currentTypeIndex++;
    } else {
        document.getElementById('pointerTypeDescription').innerHTML = "";
    }
    resetTestState();
}

function setup_pointerevent_test(testName, supportedPointerTypes) {
    return globalPointerEventTest = new MultiPointerTypeTest(testName, supportedPointerTypes);
}

function checkPointerEventType(event) {
    assert_equals(event.pointerType, expectedPointerType, "pointerType should be the same as the requested device.");
}
