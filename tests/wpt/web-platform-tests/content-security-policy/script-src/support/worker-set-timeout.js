var id = 0;
try {
    id = setTimeout("postMessage('handler invoked')", 100);
} catch (e) {}
postMessage(id === 0 ? "setTimeout blocked" : "setTimeout allowed");
