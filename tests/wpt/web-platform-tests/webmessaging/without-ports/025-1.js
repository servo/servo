var ch = new MessageChannel();
var pass1 = false;
var pass2 = false;
try {
    pass1 = ch.port1 instanceof MessagePort;
    try { var p = new MessagePort(); } catch(e) { pass2 = e instanceof TypeError; }
} catch (e) {
    ;
}
postMessage([pass1, pass2]);
