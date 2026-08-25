var QueryParser = {};

QueryParser.parseQuery = function () {
  var queryParameters = {};
  var keysAndValues = location.search.replace("?", "").split("&");
  for (var i = 0; i < keysAndValues.length; i++) {
    var key = keysAndValues[i].split("=")[0];
    var value = keysAndValues[i].split("=")[1];
    queryParameters[key] = value;
  }
  return queryParameters;
};
