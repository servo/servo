onerror = function() {
  throw new Error('Throw in error handler');
  return false;
};
onmessage = function() {
  throw new Error('Throw in message handler');
  return false;
};

if (self.location.href.indexOf(
        'throw-in-worker-initialization') >= 0) {
  throw new Error('Throw in worker initialization');
}

if (self.location.href.indexOf(
        'throw-in-setTimeout-function') >= 0) {
  // To test the behavior of setTimeout(), raw setTimeout() is used.
  setTimeout(() => { throw new Error('Throw in setTimeout function') }, 0);
}

if (self.location.href.indexOf(
        'throw-in-setTimeout-string') >= 0) {
  // To test the behavior of setTimeout(), raw setTimeout() is used.
  setTimeout("throw new Error('Throw in setTimeout string')", 0);
}
