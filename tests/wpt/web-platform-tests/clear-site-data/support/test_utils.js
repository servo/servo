var TestUtils = (function() {
  function randomString() {
    var result = "";
    for (var i = 0; i < 5; i++)
        result += String.fromCharCode(97 + Math.floor(Math.random() * 26));
    return result;
  };

  /**
   * Representation of one datatype.
   * @typedef Datatype
   * @type{object}
   * @property{string} name Name of the datatype.
   * @method{function():Void} add A function to add an instance of the datatype.
   * @method{function():boolean} isEmpty A function that tests whether
   *     the datatype's storage backend is empty.
   */
  var Datatype;

  var TestUtils = {};

  /**
   * All datatypes supported by Clear-Site-Data.
   * @param{Array.<Datatype>}
   */
  TestUtils.DATATYPES = [
    {
      "name": "cookies",
      "add": function() {
        return new Promise(function(resolve, reject) {
          document.cookie = randomString() + "=" + randomString();
          resolve();
        });
      },
      "isEmpty": function() {
        return new Promise(function(resolve, reject) {
          resolve(!document.cookie);
        });
      }
    },
    {
      "name": "storage",
      "add": function() {
        return new Promise(function(resolve, reject) {
          localStorage.setItem(randomString(), randomString());
          resolve();
        });
      },
      "isEmpty": function() {
        return new Promise(function(resolve, reject) {
          resolve(!localStorage.length);
        });
      }
    }
  ];

  /**
   * All possible combinations of datatypes.
   * @property {Array.<Array.<Datatype>>}
   */
  TestUtils.COMBINATIONS = (function() {
    var combinations = [];
    for (var mask = 0; mask < (1 << TestUtils.DATATYPES.length); mask++) {
      var combination = [];

      for (var datatype = 0;
           datatype < TestUtils.DATATYPES.length; datatype++) {
        if (mask & (1 << datatype))
          combination.push(TestUtils.DATATYPES[datatype]);
      }

      combinations.push(combination);
    }
    return combinations;
  })();

  /**
   * Get the support server URL that returns a Clear-Site-Data header
   * to clear |datatypes|.
   * @param{Array.<Datatype>} datatypes The list of datatypes to be deleted.
   * @return string The URL to be queried.
   */
  TestUtils.getClearSiteDataUrl = function(datatypes) {
    names = datatypes.map(function(e) { return e.name });
    return "support/echo-clear-site-data.py?" + names.join("&");
  }

  return TestUtils;
})();
