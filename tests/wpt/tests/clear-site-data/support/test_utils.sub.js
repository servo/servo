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
   * @property{function():boolean} supported
   *     Whether this datatype is supported by this user agent.
   * @method{function():Void} add A function to add an instance of the datatype.
   * @method{function():boolean} isEmpty A function that tests whether
   *     the datatype's storage backend is empty.
   */
  var Datatype;

  var TestUtils = {};

  /**
   * Various storage backends that are part of the 'storage' datatype.
   * @param{Array.<Datatype>}
   */
  TestUtils.STORAGE = [
    {
      "name": "local storage",
      "supported": function() { return !!window.localStorage; },
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
    },
    {
      "name": "Indexed DB",
      "supported": function() { return !!window.indexedDB; },
      "add": function() {
        return new Promise(function(resolve, reject) {
          var request = window.indexedDB.open("database");
          request.onupgradeneeded = function() {
            request.result.createObjectStore("store");
          };
          request.onsuccess = function() {
            request.result.close();
            resolve();
          }
        });
      },
      "isEmpty": function() {
        return new Promise(function(resolve, reject) {
          var request = window.indexedDB.open("database");
          request.onsuccess = function() {
            var database = request.result;
            try {
              var transaction = database.transaction(["store"]);
              resolve(false);
            } catch(error) {
              // The database is empty. However, by testing that, we have also
              // created it, which means that |onupgradeneeded| in the "add"
              // method will not run the next time. Delete the database before
              // reporting that it was empty.
              var deletion = window.indexedDB.deleteDatabase("database");
              deletion.onsuccess = resolve.bind(this, true);
            } finally {
              database.close();
            }
          };
        });
      }
    },
    {
      // TODO(@msramek): We should also test the PERSISTENT filesystem, however,
      // that might require storage permissions.
      "name": "filesystems",
      "supported": function() {
        return window.requestFileSystem || window.webkitRequestFileSystem;
      },
      "add": function() {
        return new Promise(function(resolve, reject) {
          var onSuccess = function(fileSystem) {
            fileSystem.root.getFile('file', {"create": true}, resolve, resolve);
          }
          var onFailure = resolve;

          var requestFileSystem =
              window.requestFileSystem || window.webkitRequestFileSystem;
          requestFileSystem(window.TEMPORARY, 1 /* 1B */,
                            onSuccess, onFailure);
        });
      },
      "isEmpty": function() {
        return new Promise(function(resolve, reject) {
          var onSuccess = function(fileSystem) {
            fileSystem.root.getFile(
                'file', {},
                resolve.bind(this, false) /* opened successfully */,
                resolve.bind(this, true) /* failed to open */);
          }
          var onFailure = resolve.bind(this, true);

          var requestFileSystem =
              window.requestFileSystem || window.webkitRequestFileSystem;
          requestFileSystem(window.TEMPORARY, 1 /* 1B */,
                            onSuccess, onFailure);
        });
      }
    },
    {
      "name": "service workers",
      "supported": function() { return !!navigator.serviceWorker; },
      "add": function() {
        return navigator.serviceWorker.register(
            "support/service_worker.js",
            { scope: "support/page_using_service_worker.html"});
      },
      "isEmpty": function() {
        return new Promise(function(resolve, reject) {
          navigator.serviceWorker.getRegistrations()
              .then(function(registrations) {
                resolve(!registrations.length);
              });
        });
      }
    },
    {
      "name": "Storage Buckets",
      "supported": function() { return !!navigator.storageBuckets; },
      "add": function() {
        return navigator.storageBuckets.open('inbox_bucket');
      },
      "isEmpty": function() {
        return new Promise(async function(resolve, reject) {
          var keys = await navigator.storageBuckets.keys();
          resolve(!keys.includes('inbox_bucket'));
        });
      }
    },
  ].filter(function(backend) { return backend.supported(); });

  /**
   * All datatypes supported by Clear-Site-Data.
   * @param{Array.<Datatype>}
   */
  TestUtils.DATATYPES = [
    {
      "name": "cookies",
      "supported": function() { return typeof document.cookie == "string"; },
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
      "supported": TestUtils.STORAGE[0].supported,
      "add": TestUtils.STORAGE[0].add,
      "isEmpty": TestUtils.STORAGE[0].isEmpty,
    }
  ].filter(function(datatype) { return datatype.supported(); });

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
   * Populates |datatypes| by calling the "add" method on each of them,
   * and verifies that they are nonempty.
   * @param {Array.<Datatype>} datatypes to be populated.
   * @private
   */
  function populate(datatypes) {
    return Promise.all(datatypes.map(function(datatype) {
      return new Promise(function(resolve, reject) {
        datatype.add().then(function() {
          datatype.isEmpty().then(function(isEmpty) {
            assert_false(
                isEmpty,
                datatype.name +
                    " has to be nonempty before the test starts.");
            resolve();
          });
        });
      });
    }));
  };

  /**
   * Ensures that all datatypes are nonempty. Should be called in the test
   * setup phase.
   */
  TestUtils.populateDatatypes = populate.bind(this, TestUtils.DATATYPES);

  /**
   * Ensures that all backends of the "storage" datatype are nonempty. Should
   * be called in the test setup phase.
   */
  TestUtils.populateStorage = populate.bind(this, TestUtils.STORAGE);

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

  /**
   * @param{string} page_scheme Scheme of the page. "http" or "https".
   * @param{string} resource_scheme Scheme of the resource. "http" or "https".
   * @return The URL of a page that contains a resource requesting the deletion
   *     of storage.
   */
  TestUtils.getPageWithResourceUrl = function(page_scheme, resource_scheme) {
      if (page_scheme != "https" && page_scheme != "http")
        throw "Unsupported scheme: " + page_scheme;
      if (resource_scheme != "https" && resource_scheme != "http")
        throw "Unsupported scheme: " + resource_scheme;
      return page_scheme + "://{{domains[]}}:" +
          (page_scheme == "https" ? {{ports[https][0]}} : {{ports[http][0]}}) +
          "/clear-site-data/support/page_with_resource.sub.html?scheme=" +
          resource_scheme;
  }

  return TestUtils;
})();
