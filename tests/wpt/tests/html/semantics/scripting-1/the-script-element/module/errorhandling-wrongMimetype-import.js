import foo from "./errorhandling-wrongMimetype.js?pipe=header(Content-Type,text/plain)";

// We don't expect this code to run, the import above should fail!
// If we do run though, don't trigger an error that the testharness
// might misinterpret as the import itself failing to load.

var A = null;
export { A };
