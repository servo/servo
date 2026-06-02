(function() {
  window.getSomeString = function() {
    return "œæ¹¿Ÿ"; //<- these are five Polish letters, similar to scazz. It can be read correctly only with windows 1250 encoding.
  };
})();
