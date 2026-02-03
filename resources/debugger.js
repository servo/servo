if ("dbg" in this) {
    throw new Error("Debugger script must not run more than once!");
}

const dbg = new Debugger;
const debuggeesToPipelineIds = new Map;
const debuggeesToWorkerIds = new Map;
const sourceIdsToScripts = new Map;

dbg.uncaughtExceptionHook = function(error) {
    console.error(`[debugger] Uncaught exception at ${error.fileName}:${error.lineNumber}:${error.columnNumber}: ${error.name}: ${error.message}`);
};

dbg.onNewScript = function(script, /* undefined; seems to be `script.global` now */ global) {
    // TODO: handle wasm (`script.source.introductionType == wasm`)
    sourceIdsToScripts.set(script.source.id, script);
    notifyNewSource({
        pipelineId: debuggeesToPipelineIds.get(script.global),
        workerId: debuggeesToWorkerIds.get(script.global),
        spidermonkeyId: script.source.id,
        url: script.source.url,
        urlOverride: script.source.displayURL,
        text: script.source.text,
        introductionType: script.source.introductionType ?? null,
    });
};

addEventListener("addDebuggee", event => {
    const {global, pipelineId: {namespaceId, index}, workerId} = event;
    const debuggerObject = dbg.addDebuggee(global);
    debuggeesToPipelineIds.set(debuggerObject, {
        namespaceId,
        index,
    });
    debuggeesToWorkerIds.set(debuggerObject, workerId);
});

// Create a result value object from a debuggee value.
// Debuggee values: <https://firefox-source-docs.mozilla.org/js/Debugger/Conventions.html#debuggee-values>
// Type detection follows Firefox's createValueGrip pattern:
// <https://searchfox.org/mozilla-central/source/devtools/server/actors/object/utils.js#116>
function createValueResult(value) {
    switch (typeof value) {
        case "undefined":
            return { valueType: "undefined" };
        case "boolean":
            return { valueType: "boolean", booleanValue: value };
        case "number":
            return { valueType: "number", numberValue: value };
        case "string":
            return { valueType: "string", stringValue: value };
        case "object":
            if (value === null) {
                return { valueType: "null" };
            }
            // Debugger.Object - use the `class` accessor property
            // <https://firefox-source-docs.mozilla.org/js/Debugger/Debugger.Object.html>
            return { valueType: "object", objectClass: value.class };
        default:
            return { valueType: "string", stringValue: String(value) };
    }
}

// <https://firefox-source-docs.mozilla.org/js/Debugger/Debugger.Object.html#executeinglobal-code-options>
addEventListener("eval", event => {
    const {code, pipelineId: {namespaceId, index}, workerId} = event;
    let object = debuggeesToPipelineIds.keys().next().value;
    let completionValue = object.executeInGlobal(code);

    // Completion values: <https://firefox-source-docs.mozilla.org/js/Debugger/Conventions.html#completion-values>
    let resultValue;

    if (completionValue === null) {
        resultValue = { completionType: "terminated", valueType: "undefined" };
    } else if ("throw" in completionValue) {
        // Adopt the value to ensure proper Debugger ownership
        // <https://firefox-source-docs.mozilla.org/js/Debugger/Debugger.html#adoptdebuggeevalue-value>
        // <https://searchfox.org/firefox-main/source/devtools/server/actors/webconsole/eval-with-debugger.js#312>
        // we probably don't need adoptDebuggeeValue, as we only have one debugger instance for now
        // let value = dbg.adoptDebuggeeValue(completionValue.throw);
        resultValue = { completionType: "throw", ...createValueResult(completionValue.throw) };
    } else if ("return" in completionValue) {
        // let value = dbg.adoptDebuggeeValue(completionValue.return);
        resultValue = { completionType: "return", ...createValueResult(completionValue.return) };
    }

    evalResult(event, resultValue);
});

addEventListener("getPossibleBreakpoints", event => {
    const {spidermonkeyId} = event;
    const script = sourceIdsToScripts.get(spidermonkeyId);
    let result = [];

    function getPossibleBreakpointsRecursive(script) {
        for (const location of script.getPossibleBreakpoints()) {
            location["scriptId"] = script.sourceStart;
            result.push(location);
        }
        for (const child of script.getChildScripts()) {
            getPossibleBreakpointsRecursive(child);
        }
    }
    getPossibleBreakpointsRecursive(script);

    getPossibleBreakpointsResult(event, result);
});

addEventListener("setBreakpoint", event => {
    const {spidermonkeyId, scriptId, offset} = event;
    const script = sourceIdsToScripts.get(spidermonkeyId);

    // <https://firefox-source-docs.mozilla.org/js/Debugger/Conventions.html#resumption-values>
    function breakpointHandler(...args) {
        // TODO: notify script to pause
        // tell spidermonkey to pause
       return {throw: "1"}
    }

    function setBreakpointRecursive(script) {
        if (script.sourceStart == scriptId) {
            script.setBreakpoint(offset, { hit: breakpointHandler });
            return;
        }
        for (const child of script.getChildScripts()) {
            setBreakpointRecursive(child);
        }
    }
    setBreakpointRecursive(script);
});

// <https://firefox-source-docs.mozilla.org/js/Debugger/Debugger.Frame.html>
addEventListener("pause", event => {
    dbg.onEnterFrame = function(frame) {
        dbg.onEnterFrame = undefined;
        // TODO: Some properties throw if terminated is true
        // TODO: Check if start line / column is correct or we need the proper breakpoint
        let result = {
           // TODO: arguments: frame.arguments,
           column: frame.script.startColumn,
           displayName: frame.script.displayName,
           line: frame.script.startLine,
           onStack: frame.onStack,
           oldest: frame.older == null,
           terminated: frame.terminated,
           type_: frame.type,
           url: frame.script.url,
        };
        getFrameResult(event, result);
    };
});

// <https://firefox-source-docs.mozilla.org/js/Debugger/Debugger.Script.html#clearbreakpoint-handler-offset>
// There may be more than one breakpoint at the same offset with different handlers, but we don’t handle that case for now.
addEventListener("clearBreakpoint", event => {
    const {spidermonkeyId, scriptId, offset} = event;
    const script = sourceIdsToScripts.get(spidermonkeyId);

    function setClearBreakpointRecursive(script) {
        if (script.sourceStart == scriptId) {
            // <https://firefox-source-docs.mozilla.org/js/Debugger/Debugger.Script.html#clearallbreakpoints-offset>
            // If the instance refers to a JSScript, remove all breakpoints set in this script at that offset.
            script.clearAllBreakpoints(offset);
            return;
        }
        for (const child of script.getChildScripts()) {
            setClearBreakpointRecursive(child);
        }
    }
    setClearBreakpointRecursive(script);
});
