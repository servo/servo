// adapted from http://html5demos.com/worker

var running = false;

onmessage = function (event) {
  // doesn't matter what the message is, just toggle the worker
  if (running == false) {
    running = true;
    run(1);
  } else {
    running = false;
  }
};

function run(n) {
  // split the task into 20k chunks
  var limit = n + 20000;
  search: while (running && n < limit) {
    n += 1;
    for (var i = 2; i <= Math.sqrt(n); i += 1) {
      if (n % i == 0) {
        continue search;
      }
    }
    // found a prime!
    postMessage(n);
  }
  if (n === limit) {
    // wait for the UI thread to update itself
    setTimeout(function(start_time) {
      // resume prime computation at n
      run(n);
    }, 150);
  }
}