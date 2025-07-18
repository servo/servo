if (!("dbg" in this)) {
    dbg = new Debugger;

    dbg.onNewGlobalObject = function(global) {
        console.log("[debugger] onNewGlobalObject");
        console.log(this, global);
    };

    dbg.onNewScript = function(script, global) {
        console.log("[debugger] onNewScript");
        console.log(this, script, global);
    };
}

addEventListener("addDebuggee", event => {
    const {global} = event;
    console.log("[debugger] Adding debuggee");
    dbg.addDebuggee(global);
    console.log("[debugger] getDebuggees().length =", dbg.getDebuggees().length);
});
