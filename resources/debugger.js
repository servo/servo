if (!("dbg" in this)) {
    dbg = new Debugger;

    dbg.onNewGlobalObject = function(global) {
    };

    dbg.onNewScript = function(script, global) {
    };
}

addEventListener("addDebuggee", event => {
    const {global} = event;
    dbg.addDebuggee(global);
});
