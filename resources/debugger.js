if (!("dbg" in this)) {
    dbg = new Debugger;
    debuggeesToPipelineIds = new Map;

    dbg.onNewGlobalObject = function(global) {
        console.log("[debugger] onNewGlobalObject", this, global);
    };

    dbg.onNewScript = function(script, /* undefined; seems to be `script.global` now */ global) {
        try {
            console.log("[debugger] onNewScript url=", script.url, "source id=", script.source.id, "introductionType=", script.source.introductionType);
            try {
                console.log("[debugger] source binary=", typeof script.source.binary);
            } catch (error) {
                // Do nothing; the source is not wasm
            }
            notifyNewSource({
                pipelineId: debuggeesToPipelineIds.get(script.global),
                spidermonkeyId: script.source.id,
                url: script.source.url,
                text: script.source.text,
            });
        } catch (error) {
            logError(error);
        }
    };
}

console.log("[debugger] Executing");

if ("debuggee" in this) {
    console.log("[debugger] Adding debuggee");
    const debuggerObject = dbg.addDebuggee(debuggee);
    debuggeesToPipelineIds.set(debuggerObject, {
        namespaceId: pipelineNamespaceId,
        index: pipelineIndex,
    });
}

function logError(error) {
    console.log(`[debugger] ERROR at ${error.fileName}:${error.lineNumber}:${error.columnNumber}: ${error.name}: ${error.message}`);
}
