if ("dbg" in this) {
    throw new Error("Debugger script must not run more than once!");
}

const dbg = new Debugger;
const debuggeesToPipelineIds = new Map;
const debuggeesToWorkerIds = new Map;
const sourceIdsToScripts = new Map;
const frameActorsToFrames = new Map;
const environmentActorsToEnvironments = new Map;
const environmentsToEnvironmentActors = new Map;
const blackboxing = new Map;
let suspendedFrame = null;
let lastPauseLocation = null;
let debuggerPaused = false;

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


// Convert debuggee value to property descriptor value
// <https://searchfox.org/firefox-main/source/devtools/server/actors/object/utils.js#116>
function createValueGrip(value, depth) {
    switch (typeof value) {
        case "undefined":
            return "VoidValue";
        case "boolean":
            return { BooleanValue: value };
        case "number":
            if (value === Infinity) {
                return { NumberValue: "Infinity" };
            } else if (value === -Infinity) {
                return { NumberValue: "-Infinity" };
            } else if (Number.isNaN(value)) {
                return { NumberValue: "NaN" };
            } else if (Object.is(value, -0)) {
                return { NumberValue: "-0" };
            }
            return { NumberValue: value };
        case "string":
            return { StringValue: value };
        case "object":
            // <https://searchfox.org/firefox-main/source/devtools/server/actors/object/utils.js#153>
            if (value === null) {
                return "NullValue";
            }
            if (value.optimizedOut || value.uninitialized || value.missingArguments) {
                return "NullValue";
            }
            // TODO: handle typed arrays and storage independently
            const ownPropertyLength = value.getOwnPropertyNamesLength();
            const objectValue = {
                class: value.class,
                ownPropertyLength: Number.isFinite(ownPropertyLength) ? ownPropertyLength : undefined,
            };
            // Debugger.Object - get preview using registered previewers
            // <https://firefox-source-docs.mozilla.org/devtools-user/debugger-api/debugger.object/index.html>
            const preview = getPreview(value, depth + 1);
            if (preview) {
                objectValue.preview = preview;
            }
            return { ObjectValue: objectValue };
        default:
            return { StringValue: String(value) };
    }
}

// Extract own properties from a debuggee object
// <https://firefox-source-docs.mozilla.org/devtools-user/debugger-api/debugger.object/index.html#function-properties-of-the-debugger-object-prototype>
function extractOwnProperties(obj, depth) {
    const ownProperties = [];
    let totalLength = 0;

    let names;
    try {
        names = obj.getOwnPropertyNames();
        totalLength = names.length;
    } catch (e) {
        return { ownProperties, ownPropertiesLength: 0 };
    }

    for (const name of names) {
        try {
            const desc = obj.getOwnPropertyDescriptor(name);
            if (desc) {
                const prop = {
                    name: name,
                    configurable: desc.configurable ?? false,
                    enumerable: desc.enumerable ?? false,
                    writable: desc.writable ?? false,
                    isAccessor: desc.get !== undefined || desc.set !== undefined,
                    value: createValueGrip(undefined, depth + 1),
                };

                if (desc.value !== undefined) {
                    prop.value = createValueGrip(desc.value, depth + 1);
                } else if (desc.get) {
                    try {
                        const result = desc.get.call(obj);
                        if (result && "return" in result) {
                            prop.value = createValueGrip(result.return, depth + 1);
                        }
                    } catch (e) { }
                }

                ownProperties.push(prop);
            }
        } catch (e) {
            // For now skip properties that throw on access
        }
    }

    return { ownProperties, ownPropertiesLength: totalLength };
}

// <https://searchfox.org/mozilla-central/source/devtools/server/actors/object/previewers.js#80>
const previewers = {};

// <https://searchfox.org/firefox-main/source/devtools/shared/DevToolsUtils.js#182>
function getProperty(object, name) {
    const root = object;
    while (object) {
        let desc;
        try {
            desc = object.getOwnPropertyDescriptor(name);
        } catch (e) {
            return undefined;
        }

        if (desc) {
            if ("value" in desc) {
                return desc.value;
            }

            if (desc.get) {
                try {
                    return desc.get.call(root)?.return;
                } catch (e) { }
            }

            return undefined;
        }

        object = object.proto;
    }

    return undefined;
}

// Calls the property with the given `name` on the given `object`, where
// `name` is a string, and `object` a Debugger.Object instance.
// <https://searchfox.org/firefox-main/source/devtools/shared/DevToolsUtils.js#943>
function callPropertyOnObject(object, name, ...args) {
    let descriptor;
    let proto = object;
    do {
        descriptor = proto.getOwnPropertyDescriptor(name);
        if (descriptor !== undefined) {
            break;
        }
        proto = proto.proto;
    } while (proto !== null);

    if (descriptor === undefined) {
        throw new Error("No such property");
    }

    const value = descriptor.value;
    if (typeof value !== "object" || value === null || !("callable" in value)) {
        throw new Error("Not a callable object.");
    }

    if (value.script !== undefined) {
        throw new Error(
            "The property isn't a native function and will execute code in the debuggee"
        );
    }

    const result = value.call(object, ...args);
    if (result === null) {
        throw new Error("Code was terminated.");
    }
    if ("throw" in result) {
        throw result.throw;
    }
    return result.return;
}

// <https://searchfox.org/firefox-main/source/devtools/shared/DevToolsUtils.js#983>
function* makeDebuggeeIterator(object) {
    while (true) {
        const nextValue = callPropertyOnObject(object, "next");
        if (getProperty(nextValue, "done")) {
            break;
        }
        yield getProperty(nextValue, "value");
    }
}

// <https://searchfox.org/mozilla-central/source/devtools/server/actors/object/previewers.js#125>
previewers.Function = [ function FunctionPreviewer(obj, depth) {
    let functionDetails = {
        name: obj.name,
        displayName: obj.displayName,
        parameterNames: obj.parameterNames ? obj.parameterNames: [],
        isAsync: obj.isAsyncFunction,
        isGenerator: obj.isGeneratorFunction,
    }

    let preview = { kind: "Object", function: functionDetails };
    if (depth > 1) {
        return undefined;
    }

    const { ownProperties, ownPropertiesLength } = extractOwnProperties(obj, depth);
    preview.ownProperties = ownProperties;
    preview.ownPropertiesLength = ownPropertiesLength;

    return preview;
} ];

// <https://searchfox.org/mozilla-central/source/devtools/server/actors/object/previewers.js#172>
previewers.Array = [ function ArrayPreviewer(obj, depth) {
    const lengthDescriptor = obj.getOwnPropertyDescriptor("length");
    const arrayLength = lengthDescriptor ? lengthDescriptor.value : 0;

    let preview = { kind: "ArrayLike", arrayLength };
    if (depth > 1) {
        return preview;
    }

    preview.items = [];
    for (let i = 0; i < arrayLength; i++) {
        try {
            const desc = obj.getOwnPropertyDescriptor(i);
            if (desc && desc.value !== undefined) {
                preview.items.push(createValueGrip(desc.value, depth + 1));
            }
        } catch (e) {
            // For now skip properties that throw on access
        }
    }

    return preview;
} ];

// <https://searchfox.org/firefox-main/source/devtools/server/actors/object/property-iterator.js#298>
function enumMapEntries(obj, depth) {
    const entries = makeDebuggeeIterator(callPropertyOnObject(obj, "entries"));
    return {
        *[Symbol.iterator]() {
            for (const entry of entries) {
                yield [
                    getProperty(entry, 0),
                    getProperty(entry, 1),
                ].map(value => createValueGrip(value, depth));
            }
        }
    };
}

// <https://searchfox.org/firefox-main/source/devtools/server/actors/object/previewers.js#450>
previewers.Map = [ function MapPreviewer(object, depth) {
    const size = getProperty(object, "size");
    if (typeof size !== "number") {
        return undefined;
    }

    let preview = { kind: "MapLike", size };
    if (depth > 1) {
        return preview;
    }

    preview.entries = [];
    try {
        for (const entry of enumMapEntries(object, depth)) {
            preview.entries.push(entry);
        }
    } catch (e) {
        return undefined;
    }

    return preview;
} ];

// Generic fallback for object previewer
// <https://searchfox.org/mozilla-central/source/devtools/server/actors/object/previewers.js#856>
previewers.Object = [ function ObjectPreviewer(obj, depth) {
    let preview = { kind: "Object" };
    if (depth > 1) {
       return undefined;
    }

    const { ownProperties, ownPropertiesLength } = extractOwnProperties(obj, depth);
    preview.ownProperties = ownProperties;
    preview.ownPropertiesLength = ownPropertiesLength;

    return preview;
} ];

function getPreview(obj, depth) {
    const className = obj.class;

    // <https://searchfox.org/mozilla-central/source/devtools/server/actors/object.js#295>
    const typePreviewers = previewers[className] || previewers.Object;
    for (const previewer of typePreviewers) {
        const result = previewer(obj, depth);
        if (result) return result;
    }

    return undefined;
}

// Evaluate some javascript code in the global context of the debuggee
// See executeInGlobal() at <https://firefox-source-docs.mozilla.org/devtools-user/debugger-api/debugger.object/index.html#function-properties-of-the-debugger-object-prototype>
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

    // Completion values: <https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#completion-values>
    let resultValue;
    if (completionValue === null) {
        resultValue = {
            value: createValueGrip(undefined, 0),
            hasException: false,
        };
    } else if ("throw" in completionValue) {
        // See adoptDebuggeeValue() in <https://firefox-source-docs.mozilla.org/devtools-user/debugger-api/debugger/index.html>
        // <https://searchfox.org/firefox-main/source/devtools/server/actors/webconsole/eval-with-debugger.js#312>
        // we probably don't need adoptDebuggeeValue, as we only have one debugger instance for now
        // let value = dbg.adoptDebuggeeValue(completionValue.throw);
        let realError = completionValue.throw.unsafeDereference();
        resultValue = {
            value: createValueGrip(completionValue.throw, 0),
            exceptionMessage: realError.message,
            hasException: true,
        };
    } else if ("return" in completionValue) {
        resultValue = {
            value: createValueGrip(completionValue.return, 0),
            hasException: false,
        };
    }

    evalResult(event, {
        serializedValue: JSON.stringify(resultValue.value),
        exceptionMessage: resultValue.hasException ? resultValue.exceptionMessage : null,
        hasException: resultValue.hasException,
    });
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
            displayName: frame.script.displayName ?? null,
            onStack: frame.onStack,
            oldest: frame.older == null,
            serializedThis: JSON.stringify(createValueGrip(frame.this, 0)),
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
    // https://searchfox.org/firefox-main/source/devtools/server/actors/thread.js#1706
    // We don't handle nested pauses correctly.  Don't try - if we're
    // paused, just continue running whatever code triggered the pause.
    if (debuggerPaused) {
        return undefined;
    }

    dbg.onEnterFrame = undefined;
    clearSteppingHooks(frame);

    // Get the pipeline ID for this debuggee
    const pipelineId = debuggeesToPipelineIds.get(frame.script.global);
    if (!pipelineId) {
        console.error("[debugger] No pipeline ID for frame's global");
        return undefined;
    }

    let frameActorId = createFrameActor(frame, pipelineId);

    // <https://github.com/mozilla-firefox/firefox/blob/63719d122f9214f37fd1d285a91897b8345b88b0/js/src/doc/Debugger/Debugger.Script.md?plain=1#L293-L303>
    const offset = frame.offset;
    const offsetMetadata = frame.script.getOffsetMetadata(offset);
    const frameOffset = {
        frameActorId,
        column: offsetMetadata.columnNumber - 1,
        line: offsetMetadata.lineNumber
    };
    lastPauseLocation = { line: offsetMetadata.lineNumber, column: offsetMetadata.columnNumber };

    const source = frame.script.source;
    if (source != null && isBlackBoxed(source.id, frameOffset.line, frameOffset.column)) {
        return undefined;
    }

    // Notify devtools and enter pause loop. This blocks until Resume.
    debuggerPaused = true;
    try {
        pauseAndRespond(
            pipelineId,
            frameOffset,
            pauseReason
        );
    } finally {
        debuggerPaused = false;
    }

    // <https://web.archive.org/web/20251212212538/https://firefox-source-docs.mozilla.org/js/Debugger/Conventions.html#resumption-values>
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
            // setBreakpoint(offset, handler) in <https://firefox-source-docs.mozilla.org/devtools-user/debugger-api/debugger.script/index.html#function-properties-of-the-debugger-script-prototype-object>
            // The hit handler receives a Debugger.Frame instance representing the currently executing stack frame.
            hit: (frame) => handlePauseAndRespond(frame, {type_: "breakpoint"})
        });
    }
});

// <https://firefox-source-docs.mozilla.org/devtools-user/debugger-api/debugger.frame/index.html>
addEventListener("interrupt", event => {
    dbg.onEnterFrame = (frame) => handlePauseAndRespond(
        frame,
        { type_: PAUSE_REASONS.INTERRUPTED, onNext: true }
    );
});

// <https://searchfox.org/firefox-main/source/devtools/server/actors/thread.js#1088>
function hasMoved(frame) {
    if (!lastPauseLocation) {
        return true;
    }
    const meta = frame.script.getOffsetMetadata(frame.offset);
    return meta.lineNumber !== lastPauseLocation.line ||
           meta.columnNumber !== lastPauseLocation.column;
}

function makeSteppingHooks(steppingType, startFrame) {
    return {
        onEnterFrame: function (frame) {
            const { onStep, onPop } = makeSteppingHooks("next", frame);
            frame.onStep = onStep;
            frame.onPop = onPop;
        },
        onStep: function () {
            const meta = this.script.getOffsetMetadata(this.offset);
            if (!meta.isBreakpoint || !hasMoved(this)) {
                return undefined;
            }
            if (this !== startFrame || meta.isStepStart) {
                return handlePauseAndRespond(this, { type_: PAUSE_REASONS.RESUME_LIMIT });
            }
        },
        onPop: function (completion) {
            this.reportedPop = true;
            suspendedFrame = this;
            attachSteppingHooks(steppingType, this);
            return undefined;
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
    let frame = dbg.getNewestFrame();
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
        // This is a temporary fix until we support async contexts.
        if (steppingType === "finish") {
            lastPauseLocation = null;
        }
        attachSteppingHooks(steppingType, frame);
    } else {
        clearSteppingHooks(frame);
    }
});

// <https://firefox-source-docs.mozilla.org/devtools-user/debugger-api/debugger.script/index.html#function-properties-of-the-debugger-script-prototype-object>
// There may be more than one breakpoint at the same offset with different handlers, but we don’t handle that case for now.
addEventListener("clearBreakpoint", event => {
    const {spidermonkeyId, scriptId, offset} = event;
    const script = sourceIdsToScripts.get(spidermonkeyId);
    const target = findScriptById(script, scriptId);
    if (target) {
        // If the instance refers to a JSScript, remove all breakpoints set in this script at that offset.
        target.clearAllBreakpoints(offset);
    }
});

// TODO: Get variables (scopes don't show if they don't have a variable)
function createEnvironmentActor(environment) {
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

    let bindingVariables = [];
    if (environment.type == "declarative") {
        bindingVariables = buildBindings(environment);
    }

    // <https://searchfox.org/firefox-main/source/devtools/server/actors/environment.js#62>
    if (environment.type == "object" || environment.type == "with") {
        info.serializedObject = JSON.stringify(createValueGrip(environment.object, 0));
    }
    info.serializedBindings = JSON.stringify(bindingVariables);

    let actor = environmentsToEnvironmentActors.get(environment);
    actor = registerEnvironmentActor(info, parent, actor);
    environmentsToEnvironmentActors.set(environment, actor);
    return actor;
}

function buildBindings(environment) {
    const bindingVariables = [];
    for (const name of environment.names()) {
        const value = environment.getVariable(name);
        const property = {
            name: name,
            configurable: false,
            enumerable: true,
            writable: !(
                value &&
                (value.optimizedOut || value.uninitialized || value.missingArguments)
            ),
            isAccessor: false,
            value: createValueGrip(value, 0),
        };

        bindingVariables.push(property);
    }
    return bindingVariables;
}

// Get a `Debugger.Environment` instance within which evaluation is taking place.
// <https://searchfox.org/firefox-main/source/devtools/server/actors/frame.js#109>
addEventListener("getEnvironment", event => {
    const {frameActorId} = event;
    frame = frameActorsToFrames.get(frameActorId);

    const actor = createEnvironmentActor(frame.environment);
    getEnvironmentResult(actor);
});

addEventListener("blackbox", event => {
    if (event.coversFullSource) {
        // Blackbox the entire source
        blackboxing.set(event.spidermonkeyId, []);
    } else {
        // Blackbox only a part of the source
        let blackbox = blackboxing.get(event.spidermonkeyId);
        if (blackbox == undefined) {
            blackbox = [];
        }

        blackbox.push({
            start: event.start(),
            end: event.end()
        });

        blackboxing.set(event.spidermonkeyId, blackbox);
    }
});

addEventListener("unblackbox", event => {
    if (event.coversFullSource) {
        // Unblackbox the entire source
        blackboxing.delete(event.spidermonkeyId);
    } else {
        // Unblackbox an earlier range of the source
        const array = blackboxing.get(event.spidermonkeyId);

        const start = event.start();
        const end = event.end();
        const index = array.findIndex(range => range.start.line === start.line
                && range.start.column === start.column
                && range.end.line === end.line
                && range.end.column === end.column
        );
        if (index !== -1) {
            array.splice(index, 1);

            // Empty arrays represent a fully blackboxed file
            // Therefore, if we just made the array empty we will need to remove it from the map
            if (array.length === 0) {
                blackboxing.delete(event.spidermonkeyId);
            }
        }
    }
});

function isBlackBoxed(spidermonkeyId, line, column) {
    const sourceBlackboxing = blackboxing.get(spidermonkeyId);

    if (sourceBlackboxing == undefined) {
        return false;
    } else if (sourceBlackboxing.length === 0) {
        // An empty array represents a fully ignored source
        return true;
    }

    for (const range of sourceBlackboxing) {
        return (range.start.line < line || (range.start.line === line && range.start.column <= column))
                && (range.end.line > line || (range.end.line === line && range.end.column >= column))
    }

    return false;
}
