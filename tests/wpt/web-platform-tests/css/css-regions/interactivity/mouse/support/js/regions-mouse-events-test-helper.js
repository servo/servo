// Timeout is 10 seconds for manual testing, 1.5 seconds for automated testing
var testTimeout = 10000;
if (window.testRunner) {
    testTimeout = 1500;
}
setup({timeout: testTimeout});

// This block is executed if running in WebKit's harness
if (window.testRunner) {
    testRunner.dumpAsText(false);
}

// Verify that CSS Regions are enabled in the browser.
// Divs will be horizontal if Regions are enabled.
// Divs will be vertical if Regions are not enabled.
function getLeftPosition(elemID) {
    return document.getElementById(elemID).getBoundingClientRect().left;
}

function mouseClick(block) {
    if(window.testRunner) {
        var elemBox = document.getElementById(block).getBoundingClientRect();
        var xStartPosition = elemBox.left + elemBox.width/2;
        var yStartPosition = elemBox.top + elemBox.height/2;
        eventSender.mouseMoveTo(xStartPosition, yStartPosition);
        eventSender.mouseDown();
        eventSender.mouseUp();
    }
}

function mouseDown(block) {
    if(window.testRunner) {
        var elemBox = document.getElementById(block).getBoundingClientRect();
        var xStartPosition = elemBox.left + elemBox.width/2;
        var yStartPosition = elemBox.top + elemBox.height/2;
        eventSender.mouseMoveTo(xStartPosition, yStartPosition);
        eventSender.mouseDown();
    }
}

function mouseUp(block) {
    if(window.testRunner) {
        var elemBox = document.getElementById(block).getBoundingClientRect();
        var xStartPosition = elemBox.left + elemBox.width/2;
        var yStartPosition = elemBox.top + elemBox.height/2;
        eventSender.mouseMoveTo(xStartPosition, yStartPosition);
        eventSender.mouseUp();
    }
}

function mouseDblClick(block) {
    if(window.testRunner) {
        var elemBox = document.getElementById(block).getBoundingClientRect();
        var xStartPosition = elemBox.left + elemBox.width/2;
        var yStartPosition = elemBox.top + elemBox.height/2;
        eventSender.mouseMoveTo(xStartPosition, yStartPosition);
        eventSender.mouseDown();
        eventSender.mouseUp();
        eventSender.mouseDown();
        eventSender.mouseUp();
    }
}

function mouseMove(block) {
    if(window.testRunner) {
        var elemBox = document.getElementById(block).getBoundingClientRect();
        var xStartPosition = elemBox.left + elemBox.width/2;
        var yStartPosition = elemBox.top + elemBox.height/2;
        eventSender.mouseMoveTo(xStartPosition, yStartPosition);
    }
}

function getBackgroundColor(elemID) {
    var foo = window.getComputedStyle(document.getElementById(elemID)).backgroundColor;
    return window.getComputedStyle(document.getElementById(elemID)).backgroundColor;
}

function mouseOver(block) {
    if(window.testRunner) {
        var elemBox = document.getElementById(block).getBoundingClientRect();
        var xStartPosition = elemBox.left + elemBox.width/2;
        var yStartPosition = elemBox.top + elemBox.height/2;
        eventSender.mouseMoveTo(xStartPosition, yStartPosition);
    }
}

function mouseOut(block) {
    if(window.testRunner) {
        var elemBox = document.getElementById(block).getBoundingClientRect();
        var xStartPosition = elemBox.left + elemBox.width/2;
        var yStartPosition = elemBox.top + elemBox.height/2;
        eventSender.mouseMoveTo(0, 0);
    }
}

function completionCallback () {
    add_completion_callback(function (allRes, status) {
        if(status.status === 0){
            //Update the message stating that tests are complete
            var msg = document.getElementById("msg");
            msg.innerHTML += "<p id='msg-complete'>Tests are complete. All results in the Details section below should PASS.</p>";
        }
    });
}