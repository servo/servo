var token = window.parent.token;

function getExpectedEvents() {
    return [
        {
            type: "open",
        },
        {
            type: "message",
            data: `Hello ${window.parent.user}!`,
        },
        {
            type: "close",
            code: 1000,
            reason: "",
            wasClean: true,
        },
    ];
}

function isEqual(expected, actual) {
    // good enough for our purposes here!
    return JSON.stringify(expected) === JSON.stringify(actual);
}

function testStep(expected, actual) {
    if (isEqual(expected, actual)) {
        document.body.className = "ok";
    } else if (isEqual(expected.slice(0, actual.length), actual)) {
        document.body.className = "test";
    } else {
        document.body.className = "ko";
    }
}

function runTest(websocket) {
    const expected = getExpectedEvents();
    var actual = [];
    websocket.addEventListener("open", ({ type }) => {
        actual.push({ type });
        testStep(expected, actual);
    });
    websocket.addEventListener("message", ({ type, data }) => {
        actual.push({ type, data });
        testStep(expected, actual);
    });
    websocket.addEventListener("close", ({ type, code, reason, wasClean }) => {
        actual.push({ type, code, reason, wasClean });
        testStep(expected, actual);
    });
}
