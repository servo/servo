var id = 0;
try {
    id = eval("1 + 2 + 3");
} catch (e) {}
postMessage(id === 0 ? "eval blocked" : "eval allowed");
