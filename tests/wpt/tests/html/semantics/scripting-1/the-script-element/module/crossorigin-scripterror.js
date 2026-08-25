export var foo = {};

// Push an event to the log indicating that the script was executed.
document._log.push("running");

// Deliberately trigger an error to test what details of the error
// the (possibly) cross-origin parent can listen to.
nonExistentMethod();
