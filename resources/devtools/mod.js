if ("dbg" in this) {
    throw new Error("Debugger script must not run more than once!");
}

const dbg = new Debugger;
const debuggeesToPipelineIds = new Map;
const debuggeesToWorkerIds = new Map;
const sourceIdsToScripts = new Map;
const frameActorsToFrames = new Map;
const environmentActorsToEnvironments = new Map;

// <https://searchfox.org/firefox-main/source/devtools/server/actors/thread.js#155>
// Possible values for the `why.type` attribute in "paused" event
const PAUSE_REASONS = {
  INTERRUPTED: "interrupted", // Associated with why.onNext attribute
  RESUME_LIMIT: "resumeLimit",
};

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

// Find a key by a value in a map
function findKeyByValue(map, search) {
    for (const [key, value] of map) {
        if (value === search) return key;
    }
    return undefined;
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

// Track a new debuggee global
addEventListener("addDebuggee", event => {
    const {global, pipelineId, workerId} = event;
    const debuggerObject = dbg.addDebuggee(global);
    debuggeesToPipelineIds.set(debuggerObject, pipelineId);
    if (workerId !== undefined) {
        debuggeesToWorkerIds.set(debuggerObject, workerId);
    }
});

// Maximum number of properties to include in preview
// <https://searchfox.org/firefox-main/source/devtools/server/actors/object/previewers.js#29>
const OBJECT_PREVIEW_MAX_ITEMS = 10;

// <https://searchfox.org/mozilla-central/source/devtools/server/actors/object/previewers.js#80>
const previewers = {
    Function: [],
    Array: [],
    Object: [],
    // TODO: Add Map, FormData etc
};

// Convert debuggee value to property descriptor value
// <https://searchfox.org/mozilla-central/source/devtools/server/actors/object/utils.js#116>
function valueToPropertyValue(val) {
    switch (typeof val) {
        case "undefined":
            return { valueType: "undefined" };
        case "boolean":
            return { valueType: "boolean", booleanValue: val };
        case "number":
            return { valueType: "number", numberValue: val };
        case "string":
            return { valueType: "string", stringValue: val };
        case "object":
            if (val === null) {
                return { valueType: "null" };
            }
            return {
                valueType: "object",
                objectClass: val.class,
                valueName: val.callable ? val.name : undefined,
            };
        default:
            return { valueType: "string", stringValue: String(val) };
    }
}

// Extract own properties from a debuggee object
// <https://firefox-source-docs.mozilla.org/devtools-user/debugger-api/debugger.object/index.html#function-properties-of-the-debugger-object-prototype>
function extractOwnProperties(obj, maxItems = OBJECT_PREVIEW_MAX_ITEMS) {
    const ownProperties = [];
    let totalLength = 0;

    let names;
    try {
        names = obj.getOwnPropertyNames();
        totalLength = names.length;
    } catch (e) {
        return { ownProperties, ownPropertiesLength: 0 };
    }

    let count = 0;
    for (const name of names) {
        if (count >= maxItems) break;
        try {
            const desc = obj.getOwnPropertyDescriptor(name);
            if (desc) {
                const prop = {
                    name: name,
                    configurable: desc.configurable ?? false,
                    enumerable: desc.enumerable ?? false,
                    writable: desc.writable ?? false,
                    isAccessor: desc.get !== undefined || desc.set !== undefined,
                    valueType: "undefined",
                };

                if (desc.value !== undefined) {
                    Object.assign(prop, valueToPropertyValue(desc.value));
                } else if (desc.get) {
                    try {
                        const result = desc.get.call(obj);
                        if (result && "return" in result) {
                            Object.assign(prop, valueToPropertyValue(result.return));
                        }
                    } catch (e) {
                        prop.valueType = "undefined";
                    }
                }

                ownProperties.push(prop);
                count++;
            }
        } catch (e) {
            // For now skip properties that throw on access
        }
    }

    return { ownProperties, ownPropertiesLength: totalLength };
}

// <https://searchfox.org/mozilla-central/source/devtools/server/actors/object/previewers.js#125>
previewers.Function.push(function FunctionPreviewer(obj) {
    const { ownProperties, ownPropertiesLength } = extractOwnProperties(obj);

    return {
        displayName: obj.displayName,
        parameterNames: obj.parameterNames,
        isAsync: obj.isAsyncFunction,
        isGenerator: obj.isGeneratorFunction,
        ownProperties,
        ownPropertiesLength,
    };
});

// <https://searchfox.org/mozilla-central/source/devtools/server/actors/object/previewers.js#172>
// TODO: Add implementation for showing Array items
previewers.Array.push(function ArrayPreviewer(obj) {
    const lengthDescriptor = obj.getOwnPropertyDescriptor("length");
    const length = lengthDescriptor ? lengthDescriptor.value : 0;

    return {
        kind: "ArrayLike",
        arrayLength: length,
    };
});

// Generic fallback for object previewer
// <https://searchfox.org/mozilla-central/source/devtools/server/actors/object/previewers.js#856>
previewers.Object.push(function ObjectPreviewer(obj) {
    const { ownProperties, ownPropertiesLength } = extractOwnProperties(obj);
    return {
        ownProperties,
        ownPropertiesLength,
    };
});

function getPreview(obj) {
    const className = obj.class;

    // <https://searchfox.org/mozilla-central/source/devtools/server/actors/object.js#295>
    const typePreviewers = previewers[className] || previewers.Object;
    for (const previewer of typePreviewers) {
        const result = previewer(obj);
        if (result) return result;
    }

    return { ownProperties: [], ownPropertiesLength: 0 };
}

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
            // Debugger.Object - get preview using registered previewers
            // <https://firefox-source-docs.mozilla.org/js/Debugger/Debugger.Object.html>
            const preview = getPreview(value);
            return {
                valueType: "object",
                objectClass: value.class,
                name: value.name,
                ...preview,
            };
        default:
            return { valueType: "string", stringValue: String(value) };
    }
}

// Evaluate some javascript code in the global context of the debuggee
// <https://firefox-source-docs.mozilla.org/js/Debugger/Debugger.Object.html#executeinglobal-code-options>
addEventListener("eval", event => {
    const {code, pipelineId, workerId, frameActorId} = event;

    let completionValue;
    if (frameActorId) {
        const frame = frameActorsToFrames.get(frameActorId);
        // <https://searchfox.org/firefox-main/source/js/src/doc/Debugger/Debugger.Frame.md#223>
        if (frame?.onStack) {
            completionValue = frame.eval(code);
        } else {
            completionValue = { throw: "Frame not available" };
        }
    } else {
        const object = workerId !== undefined ?
            findKeyByValue(debuggeesToWorkerIds, workerId) :
            findKeyByValue(debuggeesToPipelineIds, pipelineId);
        completionValue = object.executeInGlobal(code);
    }

    // Completion values: <https://firefox-source-docs.mozilla.org/js/Debugger/Conventions.html#completion-values>
    let resultValue;
    if (completionValue === null) {
        resultValue = { completionType: "terminated", valueType: "undefined", hasException: false };
    } else if ("throw" in completionValue) {
        // <https://firefox-source-docs.mozilla.org/js/Debugger/Debugger.html#adoptdebuggeevalue-value>
        // <https://searchfox.org/firefox-main/source/devtools/server/actors/webconsole/eval-with-debugger.js#312>
        // we probably don't need adoptDebuggeeValue, as we only have one debugger instance for now
        // let value = dbg.adoptDebuggeeValue(completionValue.throw);
        resultValue = { completionType: "throw", ...createValueResult(completionValue.throw), hasException: true };
    } else if ("return" in completionValue) {
        resultValue = { completionType: "return", ...createValueResult(completionValue.return), hasException: false };
    }

    evalResult(event, resultValue);
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

function createFrameActor(frame, pipelineId) {
    let frameActorId = findKeyByValue(frameActorsToFrames, frame);
    if (!frameActorId) {
        // TODO: Check if we already have an actor for this frame
        frameActorId = registerFrameActor(pipelineId, {
            // TODO: Some properties throw if terminated is true
            // TODO: arguments: frame.arguments,
            displayName: frame.script.displayName,
            onStack: frame.onStack,
            oldest: frame.older == null,
            terminated: frame.terminated,
            type_: frame.type,
            url: frame.script.url,
        });

        if (!frameActorId) {
            console.error("[debugger] Couldn't create frame");
            return undefined;
        }
        frameActorsToFrames.set(frameActorId, frame);
    }

    return frameActorId;
}

function handlePauseAndRespond(frame, pauseReason) {
    dbg.onEnterFrame = undefined;
    clearSteppingHooks(frame);

    // Get the pipeline ID for this debuggee
    const pipelineId = debuggeesToPipelineIds.get(frame.script.global);
    if (!pipelineId) {
        console.error("[debugger] No pipeline ID for frame's global");
        return undefined;
    }

    let frameActorId = createFrameActor(frame, pipelineId);

    // <https://firefox-source-docs.mozilla.org/js/Debugger/Debugger.Script.html#getoffsetmetadata-offset>
    const offset = frame.offset;
    const offsetMetadata = frame.script.getOffsetMetadata(offset);
    const frameOffset = {
        frameActorId,
        column: offsetMetadata.columnNumber - 1,
        line: offsetMetadata.lineNumber
    };

    // Notify devtools and enter pause loop. This blocks until Resume.
    pauseAndRespond(
        pipelineId,
        frameOffset,
        pauseReason
    );

    // <https://firefox-source-docs.mozilla.org/js/Debugger/Conventions.html#resumption-values>
    // Return undefined to continue execution normally after resume.
    return undefined;
}

addEventListener("frames", event => {
    const {pipelineId, start, count} = event;
    let frameList = handleListFrames(pipelineId, start, count);

    listFramesResult(frameList);
})

// <https://searchfox.org/firefox-main/source/devtools/server/actors/thread.js#1425>
function handleListFrames(pipelineId, start, count) {
    let frame = dbg.getNewestFrame()

    const walkToParentFrame = () => {
        if (!frame) {
            return;
        }

        const currentFrame = frame;
        frame = null;

        if (currentFrame.older) {
            frame = currentFrame.older;
        }
    }

    let i = 0;
    while (frame && i < start) {
      walkToParentFrame();
      i++;
    }

    // Return count frames, or all remaining frames if count is not defined.
    const frames = [];
    for (; frame && (!count || i < start + count); i++, walkToParentFrame()) {
      const frameActorId = createFrameActor(frame, pipelineId);
      frames.push(frameActorId);
    }

    return frames;
}

addEventListener("setBreakpoint", event => {
    const {spidermonkeyId, scriptId, offset} = event;
    const script = sourceIdsToScripts.get(spidermonkeyId);
    const target = findScriptById(script, scriptId);
    if (target) {
        target.setBreakpoint(offset, {
            // <https://firefox-source-docs.mozilla.org/js/Debugger/Debugger.Script.html#setbreakpoint-offset-handler>
            // The hit handler receives a Debugger.Frame instance representing the currently executing stack frame.
            hit: (frame) => handlePauseAndRespond(frame, {type_: "breakpoint"})
        });
    }
});

// <https://firefox-source-docs.mozilla.org/js/Debugger/Debugger.Frame.html>
addEventListener("interrupt", event => {
    dbg.onEnterFrame = (frame) => handlePauseAndRespond(
        frame,
        { type_: PAUSE_REASONS.INTERRUPTED, onNext: true }
    );
});

function makeSteppingHooks(steppingType, startFrame) {
    return {
        onEnterFrame: (frame) => {
            const { onStep, onPop } = makeSteppingHooks("next", frame);
            frame.onStep = onStep;
            frame.onPop = onPop;
        },
        onStep: () => {
            const meta = startFrame.script.getOffsetMetadata(startFrame.offset);
            if (meta.isBreakpoint && meta.isStepStart) {
                return handlePauseAndRespond(startFrame, { type_: PAUSE_REASONS.RESUME_LIMIT });
            }
        },
        onPop: (completion) => {
            this.reportedPop = true;
            suspendedFrame = startFrame;
            if (steppingType !== "finish") {
                return handlePauseAndRespond(startFrame, completion);
            }
            attachSteppingHooks("next", startFrame);
        },
    }
}

function getNextStepFrame(frame) {
    const endOfFrame = frame.reportedPop;
    const stepFrame = endOfFrame ? frame.older : frame;
    if (!stepFrame || !stepFrame.script) {
      return null;
    }
    return stepFrame;
}

// <https://searchfox.org/firefox-main/source/devtools/server/actors/thread.js#1235>
function attachSteppingHooks(steppingType, frame) {
    if (steppingType === "finish" && frame.reportedPop) {
        steppingType = "next";
    }

    const stepFrame = getNextStepFrame(frame);
    if (!stepFrame) {
        steppingType = "step";
    }

    const { onEnterFrame, onStep, onPop } = makeSteppingHooks(
        steppingType,
        frame,
    );

    if (steppingType === "step") {
        dbg.onEnterFrame = onEnterFrame;
    }

    if (stepFrame) {
        switch (steppingType) {
            case "step":
            case "next":
                if (stepFrame.script) {
                    stepFrame.onStep = onStep;
                }
            case "finish":
                stepFrame.onPop = onPop;
                break;
        }
    }
}

function clearSteppingHooks(suspendedFrame) {
    if (suspendedFrame) {
        suspendedFrame.onStep = undefined;
        suspendedFrame.onPop = undefined;
    }
    let frame = this.youngestFrame;
    if (frame?.onStack) {
        while (frame) {
            frame.onStep = undefined;
            frame.onPop = undefined;
            frame = frame.older;
        }
    }
}

// <https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#resuming-a-thread>
addEventListener("resume", event => {
    const {resumeLimitType: steppingType, frameActorID} = event;
    let frame = dbg.getNewestFrame();
    if (frameActorID) {
        frame = frameActorsToFrames.get(frameActorID);
        if (!frame) {
            console.error("[debugger] Couldn't find frame");
        }
    }
    if (steppingType) {
        attachSteppingHooks(steppingType, frame);
    } else {
        clearSteppingHooks(frame);
    }
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

// TODO: Get variables (scopes don't show if they don't have a variable)
function createEnvironmentActor(environment) {
    let actor = findKeyByValue(environmentActorsToEnvironments, environment);

    if (!actor) {
        let info = {};
        if (environment.type == "declarative") {
            info.type_ = environment.calleeScript ? "function" : "block";
        } else {
            info.type_ = environment.type;
        }

        info.scopeKind = environment.scopeKind;

        if (environment.calleeScript) {
            info.functionDisplayName = environment.calleeScript.displayName;
        }

        let parent = null;
        if (environment.parent) {
            parent = createEnvironmentActor(environment.parent);
        }

        if (environment.type == "declarative") {
            info.bindingVariables = buildBindings(environment)
        }

        // TODO: Update this instead of registering
        actor = registerEnvironmentActor(info, parent);
        environmentActorsToEnvironments.set(actor, environment);
    }

    return actor;
}

function buildBindings(environment) {
    let bindingVar = new Map();
    for (const name of environment.names()) {
        const value = environment.getVariable(name);
        // <https://searchfox.org/firefox-main/source/devtools/server/actors/environment.js#87>
        // We should not do this, it is more of a place holder for now.
        // TODO: build and pass correct structure for this. This structure is very similar to "eval"
        bindingVar[name] = JSON.stringify(value);
    }
    return bindingVar;
}

// Get a `Debugger.Environment` instance within which evaluation is taking place.
// <https://searchfox.org/firefox-main/source/devtools/server/actors/frame.js#109>
addEventListener("getEnvironment", event => {
    const {frameActorId} = event;
    frame = frameActorsToFrames.get(frameActorId);

    const actor = createEnvironmentActor(frame.environment);
    getEnvironmentResult(actor);
});
