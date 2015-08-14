// Initial setup
//@{
var globalValue;
if (globalValue === undefined) {
    globalValue = command in defaultValues ? defaultValues[command] : "";
}
var keyPrefix = globalValue == ""
    ? "manualtest-" + command + "-"
    : "manualtest-" + command + "-" + globalValue + "-";
(function(){
    var manualTests = tests[command]
        .map(function(test) { return normalizeTest(command, test) })
        .filter(function(test) { return test[1][1] == globalValue });
    var relevantMultiTests = tests.multitest
        .map(function(test) { return normalizeTest("multitest", test) })
        .filter(function(test) {
            // We only want multitests if there's exactly one occurrence of the
            // command we're testing for, and the value is correct, and that's
            // the last command we're testing.  Some of these limitations could
            // be removed in the future.
            return test[test.length - 1][0] === command
                && test[test.length - 1][1] === globalValue;
        });

    tests = manualTests.concat(relevantMultiTests);
})();
//@}

function clearCachedResults() {
//@{
    for (var key in localStorage) {
        if (key.indexOf(keyPrefix) === 0) {
            localStorage.removeItem(key);
        }
    }
}
//@}

var numManualTests = 0;
var currentTestIdx = null;

// Make sure styleWithCss is always reset to false at the start of a test run
// (I'm looking at you, Firefox)
try { document.execCommand("stylewithcss", false, "false") } catch(e) {}

function runTests() {
//@{
    // We don't ask the user to hit a key on all tests, so make sure not to
    // claim more tests are going to be run than actually are.
    for (var i = 0; i < tests.length; i++) {
        if (localStorage.getItem(keyPrefix + JSON.stringify(tests[i])) === null) {
            numManualTests++;
        }
    }

    currentTestIdx = 0;

    var runTestsButton = document.querySelector("#tests input[type=button]");
    runTestsButton.parentNode.removeChild(runTestsButton);

    var addTestButton = document.querySelector("#tests input[type=button]");
    var input = document.querySelector("#tests label input");
    // This code actually focuses and clicks everything because for some
    // reason, anything else doesn't work in IE9 . . .
    input.value = JSON.stringify(tests[0]);
    input.focus();
    addTestButton.click();
}
//@}

function addTest() {
//@{
    var tr = doSetup("#tests table", 0);
    var input = document.querySelector("#tests label input");
    var test = JSON.parse(input.value);
    doInputCell(tr, test, test.length == 2 ? command : "multitest");
    doSpecCell(tr, test, test.length == 2 ? command : "multitest");
    if (localStorage.getItem(keyPrefix + JSON.stringify(test)) !== null) {
        // Yay, I get to cheat.  Remove the overlay div so the user doesn't
        // keep hitting the key, in case it takes a while.
        var browserCell = document.createElement("td");
        tr.appendChild(browserCell);
        browserCell.innerHTML = localStorage[keyPrefix + JSON.stringify(test)];
        doBrowserCellButton(browserCell, test);
        document.getElementById("overlay").style.display = "";
        doSameCell(tr);
        runNextTest(test);
    } else {
        doBrowserCell(tr, test, function() {
            doSameCell(tr);
            runNextTest();
        });
    }
}
//@}

function runNextTest() {
//@{
    doTearDown();
    var input = document.querySelector("#tests label input");
    if (currentTestIdx === null
    || currentTestIdx + 1 >= tests.length) {
        currentTestIdx = null;
        document.getElementById("overlay").style.display = "";
        input.value = "";
        return;
    }
    currentTestIdx++;
    input.value = JSON.stringify(tests[currentTestIdx]);
    input.focus();
    addTest();
}
//@}

function doBrowserCell(tr, test, callback) {
//@{
    var browserCell = document.createElement("td");
    tr.appendChild(browserCell);

    try {
        var points = setupCell(browserCell, test[0]);

        var testDiv = browserCell.firstChild;
        // Work around weird Firefox bug:
        // https://bugzilla.mozilla.org/show_bug.cgi?id=649138
        document.body.appendChild(testDiv);
        testDiv.onkeyup = function() {
            continueBrowserCell(test, testDiv, browserCell);
            callback();
        };
        testDiv.contentEditable = "true";
        testDiv.spellcheck = false;
        if (currentTestIdx === null) {
            document.getElementById("testcount").style.display = "none";
        } else {
            document.getElementById("testcount").style.display = "";
            document.querySelector("#testcount > span").textContent = numManualTests;
            numManualTests--;
        }
        document.getElementById("overlay").style.display = "block";
        testDiv.focus();
        setSelection(points[0], points[1], points[2], points[3]);
        // Execute any extra commands beforehand, for multitests
        for (var i = 1; i < test.length - 1; i++) {
            document.execCommand(test[i][0], false, test[i][1]);
        }
    } catch (e) {
        browserCellException(e, testDiv, browserCell);
        callback();
    }
}
//@}

function continueBrowserCell(test, testDiv, browserCell) {
//@{
    try {
        testDiv.contentEditable = "inherit";
        testDiv.removeAttribute("spellcheck");
        var compareDiv1 = testDiv.cloneNode(true);

        if (getSelection().rangeCount) {
            addBrackets(getSelection().getRangeAt(0));
        }
        browserCell.insertBefore(testDiv, browserCell.firstChild);

        if (!browserCell.childNodes.length == 2) {
            throw "The cell didn't have two children.  Did something spill outside the test div?";
        }

        compareDiv1.normalize();
        // Sigh, Gecko is crazy
        var treeWalker = document.createTreeWalker(compareDiv1, NodeFilter.SHOW_ELEMENT, null, null);
        while (treeWalker.nextNode()) {
            var remove = [].filter.call(treeWalker.currentNode.attributes, function(attrib) {
                return /^_moz_/.test(attrib.name) || attrib.value == "_moz";
            });
            for (var i = 0; i < remove.length; i++) {
                treeWalker.currentNode.removeAttribute(remove[i].name);
            }
        }
        var compareDiv2 = compareDiv1.cloneNode(false);
        compareDiv2.innerHTML = compareDiv1.innerHTML;
        if (!compareDiv1.isEqualNode(compareDiv2)
        && compareDiv1.innerHTML != compareDiv2.innerHTML) {
            throw "DOM does not round-trip through serialization!  "
                + compareDiv1.innerHTML + " vs. " + compareDiv2.innerHTML;
        }
        if (!compareDiv1.isEqualNode(compareDiv2)) {
            throw "DOM does not round-trip through serialization (although innerHTML is the same)!  "
                + compareDiv1.innerHTML;
        }

        browserCell.lastChild.textContent = browserCell.firstChild.innerHTML;
    } catch (e) {
        browserCellException(e, testDiv, browserCell);
    }

    localStorage[keyPrefix + JSON.stringify(test)] = browserCell.innerHTML;

    doBrowserCellButton(browserCell, test);
}
//@}

function doBrowserCellButton(browserCell, test) {
//@{
    var button = document.createElement("button");
    browserCell.lastChild.appendChild(button);
    button.textContent = "Redo browser output";
    button.onclick = function() {
        localStorage.removeItem(keyPrefix + JSON.stringify(test));
        var tr = browserCell.parentNode;
        while (browserCell.nextSibling) {
            tr.removeChild(browserCell.nextSibling);
        }
        tr.removeChild(browserCell);
        doBrowserCell(tr, test, function() {
            doSameCell(tr);
            doTearDown();
            document.getElementById("overlay").style.display = "";
            tr.scrollIntoView();
        });
    };
}
//@}
// vim: foldmarker=@{,@} foldmethod=marker
