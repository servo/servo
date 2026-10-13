// META: script=resources/multiple-content-disposition.js

runMultipleContentDispositionTests(
    "HTTP/1.1", (values, name) => `resources/${name}-duplicates.asis`);
