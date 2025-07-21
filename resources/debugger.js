const dbg = new Debugger;
setInterval(() => {
    console.log(dbg.findAllGlobals());
}, 250);
