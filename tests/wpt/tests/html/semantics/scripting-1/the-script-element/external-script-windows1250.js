(function() {
  window.getSomeString = function() {
    return "�湿�"; //<- these are five Polish letters, similar to scazz. It can be read correctly only with windows 1250 encoding.
  };
})();
