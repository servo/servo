var callback = arguments[arguments.length - 1];
window.opener.testdriver_callback = callback;
window.opener.process_next_event();
