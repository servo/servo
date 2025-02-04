importScripts('./dispatcher.js');

const params = new URLSearchParams(location.search);
const uuid = params.get('uuid');
const executor = new Executor(uuid);  // `execute()` is called in constructor.
