// Perf Tests run a maximum of 20 times,
// make sure we have an equal amount of characters
// for each run.
var selectionSize = fallbackChars.length / 21;
var target;

function test() {
    var charSelection = "";
    for(var i=0; i < selectionSize; i++) {
        var selectedCharIndex = Math.floor(Math.random() * fallbackChars.length);
        if(!fallbackChars[selectedCharIndex])
            continue;
        charSelection += fallbackChars[selectedCharIndex];
        fallbackChars.splice(selectedCharIndex, 1);
    }
    if (charSelection.length)
        replaceTextAndWaitForLayout(charSelection);
}

function replaceTextAndWaitForLayout(charSelection) {
    while (target.firstChild)
        target.removeChild(target.firstChild);
    target.appendChild(document.createTextNode(charSelection));
    PerfTestRunner.forceLayout();
}

function cleanup() {
    replaceTextAndWaitForLayout("");
}

function startTest() {
    target = document.querySelector("#target");
    PerfTestRunner.measureTime({ run: test, done: cleanup, description: "Per-character font fallback" });
}
