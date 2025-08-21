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
    dbg.addDebuggee(global);
    const debuggerObject = dbg.addDebuggee(global);
    debuggeesToPipelineIds.set(debuggerObject, {
        namespaceId,
        index,
    });
    debuggeesToWorkerIds.set(debuggerObject, workerId);
});

addEventListener("getPossibleBreakpoints", event => {
    const {spidermonkeyId} = event;
    const script = sourceIdsToScripts.get(spidermonkeyId);
    function getPossibleBreakpointsRecursive(script) {
        const result = script.getPossibleBreakpoints(/* TODO: `query` */);
        for (const child of script.getChildScripts()) {
            for (const location of getPossibleBreakpointsRecursive(child)) {
                result.push(location);
            }
        }
        return result;
    }
    getPossibleBreakpointsResult(event, getPossibleBreakpointsRecursive(script));
});
