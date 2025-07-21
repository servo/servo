if (!("dbg" in this)) {
    dbg = new Debugger;

    dbg.onNewScript = function(script, global) {
        console.log("[debugger] onNewScript");
        console.log(this, script, global);
    };

    dbg.onNativeCall = function(callee, reason) {
        console.log("[debugger] onNativeCall");
        console.log(this, callee, reason);
    };

    dbg.onNewGlobalObject = function(global) {
        console.log("[debugger] onNewGlobalObject");
        console.log(this, global);
    };
}

console.log("[debugger] Executing");

if ("debuggee" in this) {
    console.log("[debugger] Adding debuggee");
    dbg.addDebuggee(debuggee);
    console.log("[debugger] getDebuggees().length =", dbg.getDebuggees().length);
}
