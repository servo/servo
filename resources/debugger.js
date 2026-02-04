if ("dbg" in this) {
    throw new Error("Debugger script must not run more than once!");
}

const dbg = new Debugger;
const debuggeesToPipelineIds = new Map;
const debuggeesToWorkerIds = new Map;
const sourceIdsToScripts = new Map;

// Find script by scriptId within a script tree
function findScriptById(script, scriptId) {
    if (script.sourceStart === scriptId) {
        return script;
    }
    for (const child of script.getChildScripts()) {
        const found = findScriptById(child, scriptId);
        if (found) return found;
    }
    return null;
}

// Walk script tree and call callback for each script
function walkScriptTree(script, callback) {
    callback(script);
    for (const child of script.getChildScripts()) {
        walkScriptTree(child, callback);
    }
}

dbg.uncaughtExceptionHook = function(error) {
    console.error(`[debugger] Uncaught exception at ${error.fileName}:${error.lineNumber}:${error.columnNumber}: ${error.name}: ${error.message}`);
};

dbg.onNewScript = function(script) {
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
    debuggeesToPipelineIds.set(debuggerObject, { namespaceId, index });
    debuggeesToWorkerIds.set(debuggerObject, workerId);
});

addEventListener("getPossibleBreakpoints", event => {
    const {spidermonkeyId} = event;
    const script = sourceIdsToScripts.get(spidermonkeyId);
    const result = [];
    walkScriptTree(script, (currentScript) => {
        for (const location of currentScript.getPossibleBreakpoints()) {
            location["scriptId"] = currentScript.sourceStart;
            result.push(location);
        }
    });
    getPossibleBreakpointsResult(event, result);
});

addEventListener("setBreakpoint", event => {
    const {spidermonkeyId, scriptId, offset} = event;
    const script = sourceIdsToScripts.get(spidermonkeyId);
    const target = findScriptById(script, scriptId);
    if (target) {
        target.setBreakpoint(offset, {
            hit: () => {
                // <https://firefox-source-docs.mozilla.org/js/Debugger/Conventions.html#resumption-values>
                // TODO: notify script to pause
                return { throw: "1" };
            }
        });
    }
});

// <https://firefox-source-docs.mozilla.org/js/Debugger/Debugger.Frame.html>
addEventListener("pause", event => {
    dbg.onEnterFrame = function(frame) {
        dbg.onEnterFrame = undefined;
        // TODO: Some properties throw if terminated is true
        // TODO: Check if start line / column is correct or we need the proper breakpoint
        const result = {
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
    const target = findScriptById(script, scriptId);
    if (target) {
        // <https://firefox-source-docs.mozilla.org/js/Debugger/Debugger.Script.html#clearallbreakpoints-offset>
        // If the instance refers to a JSScript, remove all breakpoints set in this script at that offset.
        target.clearAllBreakpoints(offset);
    }
});
