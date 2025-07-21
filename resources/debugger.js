if (!("dbg" in this)) {
    dbg = new Debugger;
}

console.log("[debugger] Executing");

if ("debuggee" in this) {
    console.log("[debugger] Adding debuggee");
    dbg.addDebuggee(debuggee);
}
