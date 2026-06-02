var _testing = false;

// The index into _keyTable of the key currently being tested.
var _currKey = 0;

var _keysTotal = 0;
var _keysGood = 0;
var _keysBad = 0;
var _keysSkipped = 0;

var _modifierMode = "None";

var _keydownCapture = [];
var _keyupCapture = [];

var CAPTURE_KEYCODE = 0;
var CAPTURE_CODE = 1;
var CAPTURE_KEY = 2;
var CAPTURE_SHIFTKEY = 3;
var CAPTURE_CONTROLKEY = 4;
var CAPTURE_ALTKEY = 5;
var CAPTURE_METAKEY = 6;

// An array of KeyInfo for each key to be tested.
var _keyTable = [];

// KeyInfo fields.
var KEYINFO_CODE = 0;       // |code| for this key
var KEYINFO_ROW = 1;        // Keyboard row
var KEYINFO_TYPE = 2;       // Key type (see below)
var KEYINFO_WIDTH = 3;      // Width of key: 0=normal
var KEYINFO_KEYCAP = 4;     // Keycap string to display
var KEYINFO_KEY = 5;        // Unmodified key value
var KEYINFO_KEY_SHIFT = 6;  // Shifted key value

var KEYTYPE_NORMAL = 0;
var KEYTYPE_DISABLED = 1;   // Key cannot be tested: e.g., CapsLock
var KEYTYPE_END = 2;        // Used to mark end of KeyTable
var KEYTYPE_MODIFIER = 3;   // Modifer key

function clearChildren(e) {
    while (e.firstChild !== null) {
        e.removeChild(e.firstChild);
    }
}

function setText(e, text) {
    clearChildren(e);
    e.appendChild(document.createTextNode(text));
}

function setUserAgent() {
    var userAgent = navigator.userAgent;
    uaDiv = document.getElementById("useragent");
    setText(uaDiv, userAgent);
}

function addEventListener(obj, etype, handler) {
    if (obj.addEventListener) {
        obj.addEventListener(etype, handler, false);
    } else if (obj.attachEvent) {
        obj.attachEvent("on"+etype, handler);
    } else {
        obj["on"+etype] = handler;
    }
}

function addClass(obj, className) {
    obj.classList.add(className);
}

function removeClass(obj, className) {
    obj.classList.remove(className);
}

function addInnerText(obj, text) {
    obj.appendChild(document.createTextNode(text));
}

function calcLocation(loc) {
    if (loc == 1) return "LEFT";
    if (loc == 2) return "RIGHT";
    if (loc == 3) return "NUMPAD";
    return loc;
}

function isModifierKey(e) {
    // Shift, Control, Alt
    if (e.keyCode >= 16 && e.keyCode <= 18) {
        return true;
    }
    // Windows, Command or Meta key.
    if (e.keyCode == 224 // Right/Left: Gecko
        || e.keyCode == 91    // Left: WebKit/Blink
        || e.keyCode == 93    // Right: WebKit/Blink
        ) {
        return true;
    }
    return false;
}

function init(title, keytable) {
    _keyTable = keytable;

    createBody(title, keytable);

    setUserAgent();

    var input = document.getElementById("input");
    input.disabled = true;
    addEventListener(input, "keydown", onKeyDown);
    addEventListener(input, "keyup", onKeyUp);
    //addEventListener(input, "beforeInput", onBeforeInput);
    //addEventListener(input, "input", onInput);
}

function onKeyDown(e) {
    // Ignore modifier keys when checking modifier combinations.
    if (_modifierMode != "None" && isModifierKey(e)) {
        return;
    }

    _keydownInfo = [e.keyCode, e.code, e.key, e.shiftKey, e.ctrlKey, e.altKey, e.metaKey];
    if (e.keyCode == 9 || e.code == "Tab") {
        e.preventDefault();
    }
}

function onKeyUp(e) {
    // Ignore modifier keys when checking modifier combinations.
    if (_modifierMode != "None" && isModifierKey(e)) {
        return;
    }

    _keyupInfo = [e.keyCode, e.code, e.key, e.shiftKey, e.ctrlKey, e.altKey, e.metaKey];

    if (_testing) {
        verifyKey();
        nextKey();
    }
}

function onBeforeInput(e) {
}

function onInput(e) {
}

function addError(elem, str) {
    var p = document.createElement('p');
    p.classList.add("error2");
    p.textContent = str;
    elem.appendChild(p);
}

function addErrorIncorrect(elem, eventName, attrName, keyEventInfo, attr, expected) {
    addError(elem, "Incorrect " + eventName
        + " |" + attrName + "| = " + keyEventInfo[attr]
        + " - Expected " + expected);
}

function verifyKeyEventFields(eventName, keyEventInfo, code, key, error) {
    var verifyCode = document.getElementById("opt_attr_code").checked;
    var verifyKey = document.getElementById("opt_attr_key").checked;
    var verifyModifiers = document.getElementById("opt_attr_modifiers").checked;
    var good = true;

    if (!verifyCode && !verifyKey && !verifyModifiers) {
        good = false;
        addError(error, "Invalid test: At least one attribute must be selected for testing.");
    }
    if (verifyCode && keyEventInfo[CAPTURE_CODE] != code) {
        good = false;
        addErrorIncorrect(error, eventName, "code", keyEventInfo, CAPTURE_CODE, code);
    }
    if (verifyKey && keyEventInfo[CAPTURE_KEY] != key) {
        good = false;
        addErrorIncorrect(error, eventName, "key", keyEventInfo, CAPTURE_KEY, key);
    }
    if (verifyModifiers) {
        if (keyEventInfo[CAPTURE_SHIFTKEY] != (_modifierMode == "Shift")) {
            good = false;
            addErrorIncorrect(error, eventName, "shiftKey", keyEventInfo, CAPTURE_SHIFTKEY, false);
        }
        if (keyEventInfo[CAPTURE_CONTROLKEY]) {
            good = false;
            addErrorIncorrect(error, eventName, "controlKey", keyEventInfo, CAPTURE_CONTROLKEY, false);
        }
        if (keyEventInfo[CAPTURE_ALTKEY]) {
            good = false;
            addErrorIncorrect(error, eventName, "altKey", keyEventInfo, CAPTURE_ALTKEY, false);
        }
        if (keyEventInfo[CAPTURE_METAKEY]) {
            good = false;
            addErrorIncorrect(error, eventName, "metaKey", keyEventInfo, CAPTURE_METAKEY, false);
        }
    }

    return good;
}

function verifyKey() {
    _keysTotal++;

    var keyInfo = _keyTable[_currKey];
    var code = keyInfo[KEYINFO_CODE];
    var key = keyInfo[KEYINFO_KEY];
    var keyShift = keyInfo[KEYINFO_KEY_SHIFT];

    var keyCheck = key;
    if (_modifierMode == "Shift") {
        keyCheck = keyShift;
    }

    var verifyKeydown = document.getElementById("opt_event_keydown").checked;
    var verifyKeyup = document.getElementById("opt_event_keyup").checked;

    var error = document.createElement('div');
    error.classList.add("error");
    var good = true;

    if (verifyKeydown) {
        good = verifyKeyEventFields("keydown", _keydownInfo, code, keyCheck, error);
    }
    if (verifyKeyup) {
        good = verifyKeyEventFields("keyup", _keyupInfo, code, keyCheck, error);
    }

    if (!verifyKeydown && !verifyKeyup) {
        good = false;
        addError(error, "Invalid test: At least one event must be selected for testing.");
    }

    // Allow Escape key to skip the current key.
    var skipped = false;
    if (_keydownInfo[CAPTURE_KEYCODE] == 27 || _keydownInfo[CAPTURE_CODE] == "Escape") {
        good = true;
        skipped = true;
    }

    if (!good) {
        var p = document.createElement('p');
        p.classList.add("error1");
        p.textContent = "Error : " + code;
        error.insertBefore(p, error.firstChild);
    }

    removeNextKeyHilight();
    if (skipped) {
        _keysSkipped++;
        document.getElementById(code).classList.add("skippedKey")
    } else if (good) {
        _keysGood++;
        document.getElementById(code).classList.add("goodKey")
    } else {
        _keysBad++;
        document.getElementById(code).classList.add("badKey")
    }
    updateTestSummary(good ? null : error);
}

function updateTestSummary(error) {
    document.getElementById("keys-total").textContent = _keysTotal;
    document.getElementById("keys-good").textContent = _keysGood;
    document.getElementById("keys-bad").textContent = _keysBad;
    document.getElementById("keys-skipped").textContent = _keysSkipped;

    if (error) {
        var errors = document.getElementById("errors");
        errors.insertBefore(error, errors.firstChild);
    }
}

function resetTest() {
    _keysTotal = 0;
    _keysGood = 0;
    _keysBad = 0;

    _currKey = -1;
    nextKey();

    updateTestSummary();

    // Remove previous test results.
    clearChildren(document.getElementById("errors"));

    // Remove highlighting from keys.
    for (var i = 0; i < _keyTable.length; i++) {
        var code = _keyTable[i][KEYINFO_CODE];
        var type = _keyTable[i][KEYINFO_TYPE];
        if (type != KEYTYPE_END) {
            var key = document.getElementById(code);
            key.classList.remove("goodKey");
            key.classList.remove("badKey");
            key.classList.remove("skippedKey");
        }
    }
}

function startTest() {
    if (_testing) {
        // Cancel the currently running test.
        endTest();
        return;
    }

    resetTest();
    _testing = true;
    document.getElementById("start").value = "Stop Test"

    var input = document.getElementById("input");
    input.value = "";
    input.disabled = false;
    input.focus();

    // Show test instructions and info.
    document.getElementById("test-info").style.display = 'block';
    document.getElementById("instructions").style.display = 'block';
    document.getElementById("test-done").style.display = 'none';
}

function endTest() {
    _testing = false;
    removeNextKeyHilight();
    document.getElementById("start").value = "Restart Test"
    document.getElementById("input").disabled = true;
    document.getElementById("instructions").style.display = 'none';
    document.getElementById("test-done").style.display = 'block';
}

function removeNextKeyHilight() {
    var curr = document.getElementById(_keyTable[_currKey][KEYINFO_CODE]);
    if (curr) {
        removeClass(curr, "nextKey")
    }
}

function addNextKeyHilight() {
    var curr = document.getElementById(_keyTable[_currKey][KEYINFO_CODE]);
    if (curr) {
        addClass(curr, "nextKey")
    }
}

function nextKey() {
    var keyInfo;
    var keepLooking = true;
    do {
        _currKey++;
        keyInfo = _keyTable[_currKey];
        var type = keyInfo[KEYINFO_TYPE];

        // Skip over disabled keys.
        keepLooking = (type == KEYTYPE_DISABLED);

        // Skip over modifier keys if we're testing modifier combinations.
        if (_modifierMode != "None" && type == KEYTYPE_MODIFIER) {
            keepLooking = true;
        }

        // Skip over keys in disabled rows.
        if (type != KEYTYPE_END) {
            var row = keyInfo[KEYINFO_ROW];
            var rowEnabled = document.getElementById("opt_row_" + row).checked;
            keepLooking = keepLooking || !rowEnabled;
        }
    } while (keepLooking);

    if (keyInfo[KEYINFO_TYPE] == KEYTYPE_END) {
        endTest();
    } else {
        addNextKeyHilight();
    }
}

function toggleOptions() {
    var link = document.getElementById("optionstoggle");
    var options = document.getElementById("options");
    clearChildren(link);
    if (options.style.display == "block") {
        options.style.display = "none";
        addInnerText(link, "Show Options");
    }
    else {
        options.style.display = "block";
        addInnerText(link, "Hide Options");
    }
}

function toggleHelp() {
    var link = document.getElementById("helptoggle");
    var help = document.getElementById("help");
    clearChildren(link);
    if (help.style.display == "block") {
        help.style.display = "none";
        addInnerText(link, "Show Help");
    }
    else {
        help.style.display = "block";
        addInnerText(link, "Hide Help");
    }
}

function createBody(title, keytable) {
    var body = document.getElementsByTagName("body")[0];
    var p;
    var span;

    var h1 = document.createElement('h1');
    h1.textContent = "Keyboard Event Manual Test - " + title;
    body.appendChild(h1);

    // Display useragent.
    p = document.createElement('p');
    p.textContent = "UserAgent: ";
    var useragent = document.createElement('span');
    useragent.id = "useragent";
    p.appendChild(useragent);
    body.appendChild(p);

    // Display input textedit.
    p = document.createElement('p');
    p.textContent = "Test Input: ";
    var input1 = document.createElement('input');
    input1.id = "input";
    input1.type = "text";
    input1.size = 80;
    p.appendChild(input1);
    p.appendChild(document.createTextNode(" "));
    var input2 = document.createElement('input');
    input2.id = "start";
    input2.type = "button";
    input2.onclick = function() { startTest(); return false; }
    input2.value = "Start Test";
    p.appendChild(input2);
    p.appendChild(document.createTextNode(" "));
    var optionsToggle = document.createElement('a');
    optionsToggle.id = "optionstoggle";
    optionsToggle.href = "javascript:toggleOptions()";
    optionsToggle.textContent = "Show Options";
    p.appendChild(optionsToggle);
    p.appendChild(document.createTextNode(" "));
    var helpToggle = document.createElement('a');
    helpToggle.id = "helptoggle";
    helpToggle.href = "javascript:toggleHelp()";
    helpToggle.textContent = "Show Help";
    p.appendChild(helpToggle);
    body.appendChild(p);

    createOptions(body);

    createHelp(body);

    createKeyboard(body, keytable);

    // Test info and summary.
    var test_info = document.createElement('div');
    test_info.id = "test-info";
    test_info.style.display = "none";

    var instructions = document.createElement('div');
    instructions.id = "instructions";
    p = document.createElement('p');
    p.textContent = "Press the highlighted key.";
    instructions.appendChild(p);
    test_info.appendChild(instructions);

    var test_done = document.createElement('div');
    test_done.id = "test-done";
    p = document.createElement('p');
    p.textContent = "Test complete!";
    test_done.appendChild(p);
    test_info.appendChild(test_done);

    var summary = document.createElement('div');
    summary.id = "summary";
    p = document.createElement('p');
    summary.appendChild(document.createTextNode("Keys Tested: "));
    span = document.createElement('span');
    span.id = "keys-total";
    span.textContent = 0;
    summary.appendChild(span);
    summary.appendChild(document.createTextNode("; Passed "));
    span = document.createElement('span');
    span.id = "keys-good";
    span.textContent = 0;
    summary.appendChild(span);
    summary.appendChild(document.createTextNode("; Failed "));
    span = document.createElement('span');
    span.id = "keys-bad";
    span.textContent = 0;
    summary.appendChild(span);
    summary.appendChild(document.createTextNode("; Skipped "));
    span = document.createElement('span');
    span.id = "keys-skipped";
    span.textContent = 0;
    summary.appendChild(span);
    test_info.appendChild(summary);

    var errors = document.createElement('div');
    errors.id = "errors";
    test_info.appendChild(errors);

    body.appendChild(test_info);
}

function addOptionTitle(cell, title) {
    var span = document.createElement('span');
    span.classList.add("opttitle");
    span.textContent = title;
    cell.appendChild(span);
    cell.appendChild(document.createElement("br"));
}

function addOptionCheckbox(cell, id, text) {
    var label = document.createElement("label");

    var input = document.createElement("input");
    input.type = "checkbox";
    input.id = id;
    input.checked = true;
    label.appendChild(input);

    label.appendChild(document.createTextNode(" " + text));
    cell.appendChild(label);

    cell.appendChild(document.createElement("br"));
}

function addOptionRadio(cell, group, text, handler, checked) {
    var label = document.createElement("label");

    var input = document.createElement("input");
    input.type = "radio";
    input.name = group;
    input.value = text;
    input.onclick = handler;
    input.checked = checked;
    label.appendChild(input);

    label.appendChild(document.createTextNode(" " + text));
    cell.appendChild(label);

    cell.appendChild(document.createElement("br"));
}

function handleModifierGroup() {
    var radio = document.querySelector("input[name=opt_modifier]:checked");
    var oldMode = _modifierMode;
    _modifierMode = radio.value;

    if (oldMode == "Shift") {
        document.getElementById("ShiftLeft").classList.remove("activeModifierKey");
        document.getElementById("ShiftRight").classList.remove("activeModifierKey");
    }

    if (_modifierMode == "Shift") {
        document.getElementById("ShiftLeft").classList.add("activeModifierKey");
        document.getElementById("ShiftRight").classList.add("activeModifierKey");
    }
}

function createOptions(body) {
    var options = document.createElement('div');
    options.id = "options";
    options.style.display = "none";

    var table = document.createElement('table');
    table.classList.add("opttable");
    var row = document.createElement('tr');
    var cell;

    cell = document.createElement('td');
    cell.classList.add("optcell");
    addOptionTitle(cell, "Keyboard Rows");
    addOptionCheckbox(cell, "opt_row_0", "Row E (top)");
    addOptionCheckbox(cell, "opt_row_1", "Row D");
    addOptionCheckbox(cell, "opt_row_2", "Row C");
    addOptionCheckbox(cell, "opt_row_3", "Row B");
    addOptionCheckbox(cell, "opt_row_4", "Row A (bottom)");
    row.appendChild(cell);

    cell = document.createElement('td');
    cell.classList.add("optcell");
    addOptionTitle(cell, "Events");
    addOptionCheckbox(cell, "opt_event_keydown", "keydown");
    addOptionCheckbox(cell, "opt_event_keyup", "keyup");
    row.appendChild(cell);

    cell = document.createElement('td');
    cell.classList.add("optcell");
    addOptionTitle(cell, "Attributes");
    addOptionCheckbox(cell, "opt_attr_code", "code");
    addOptionCheckbox(cell, "opt_attr_key", "key");
    addOptionCheckbox(cell, "opt_attr_modifiers", "modifiers");
    row.appendChild(cell);

    cell = document.createElement('td');
    cell.classList.add("optcell");
    addOptionTitle(cell, "Modifiers");
    addOptionRadio(cell, "opt_modifier", "None", handleModifierGroup, true);
    addOptionRadio(cell, "opt_modifier", "Shift", handleModifierGroup, false);
    row.appendChild(cell);

    table.appendChild(row);
    options.appendChild(table);

    body.appendChild(options);
}

function addHelpText(div, text) {
    var p = document.createElement('p');
    p.classList.add("help");
    p.textContent = text;
    div.appendChild(p);
}

function createHelp(body) {
    var help = document.createElement('div');
    help.id = "help";
    help.style.display = "none";

    addHelpText(help, "Click on the \"Start Test\" button to begin testing.");
    addHelpText(help, "Press the hilighted key to test it.");
    addHelpText(help, "Clicking anywhere outside the \"Test Input\" editbox will pause testing. To resume, click back inside the editbox.");
    addHelpText(help, "To skip a key while testing, press Escape.");
    addHelpText(help, "When testing with modifier keys, the modifier must be pressed before the keydown and released after the keyup of the key being tested.");

    body.appendChild(help);
}

function createKeyboard(body, keytable) {
    var keyboard = document.createElement('div');
    keyboard.classList.add("keyboard");

    var currRow = 0;
    var row = document.createElement('div');
    row.classList.add("key-row");

    for (var i = 0; i < keytable.length; i++) {
        var code = keytable[i][KEYINFO_CODE];
        var rowId = keytable[i][KEYINFO_ROW];
        var type = keytable[i][KEYINFO_TYPE];
        var width = keytable[i][KEYINFO_WIDTH];
        var keyCap = keytable[i][KEYINFO_KEYCAP];

        if (type == KEYTYPE_END) {
            continue;
        }

        if (rowId != currRow) {
            keyboard.appendChild(row);
            row = document.createElement('div');
            row.classList.add("key-row");
            currRow = rowId;
        }

        var key = document.createElement('div');
        key.id = code;
        key.classList.add("key");
        if (width != 0) {
            key.classList.add("wide" + width);
        }
        key.textContent = keyCap;

        row.appendChild(key);
    }

    keyboard.appendChild(row);
    body.appendChild(keyboard);
}
