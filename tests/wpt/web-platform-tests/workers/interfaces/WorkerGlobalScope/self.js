var results = [];
function check(func, msg) {
  try {
    results.push([func(), msg]);
  } catch(ex) {
    results.push([String(ex), msg]);
  }
}
check(function() { return self === self; }, 'self === self');
check(function() { return self instanceof WorkerGlobalScope; }, 'self instanceof WorkerGlobalScope');
check(function() { return 'self' in self; }, '\'self\' in self');
check(function() {
  var x = self;
  self = 1;
  return x === self;
}, 'self = 1');
postMessage(results);