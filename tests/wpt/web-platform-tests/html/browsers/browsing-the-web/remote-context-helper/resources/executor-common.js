// Functions available by default in the executor.

'use strict';

let executor;

// Expects addScript to be present (window or worker version).
function addScripts(urls) {
  return Promise.all(urls.map(addScript));
}

function startExecutor() {
  const params = new URLSearchParams(location.search);
  addScripts(params.getAll('script'));
  const uuid = params.get('uuid');
  executor = new Executor(uuid);
}
