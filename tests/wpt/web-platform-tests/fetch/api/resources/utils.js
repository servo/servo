var inWorker = false;
var RESOURCES_DIR = "../resources/";

try {
  inWorker = !(self instanceof Window);
} catch (e) {
  inWorker = true;
}

function dirname(path) {
    return path.replace(/\/[^\/]*$/, '/')
}

function checkRequest(request, ExpectedValuesDict) {
  for (var attribute in ExpectedValuesDict) {
    switch(attribute) {
      case "headers":
        for (var key in ExpectedValuesDict["headers"].keys()) {
          assert_equals(request["headers"].get(key), ExpectedValuesDict["headers"].get(key),
            "Check headers attribute has " + key + ":" + ExpectedValuesDict["headers"].get(key));
        }
        break;

      case "body":
        //for checking body's content, a dedicated asyncronous/promise test should be used
        assert_true(request["headers"].has("Content-Type") , "Check request has body using Content-Type header")
        break;

      case "method":
      case "referrer":
      case "referrerPolicy":
      case "credentials":
      case "cache":
      case "redirect":
      case "integrity":
      case "url":
      case "destination":
        assert_equals(request[attribute], ExpectedValuesDict[attribute], "Check " + attribute + " attribute")
        break;

      default:
        break;
    }
  }
}

//check reader's text content in an asyncronous test
function readTextStream(reader, asyncTest, expectedValue, retrievedText) {
  if (!retrievedText)
    retrievedText = "";
  reader.read().then(function(data) {
    if (!data.done) {
      var decoder = new TextDecoder();
      retrievedText += decoder.decode(data.value);
      readTextStream(reader, asyncTest, expectedValue, retrievedText);
      return;
    }
    asyncTest.step(function() {
      assert_equals(retrievedText, expectedValue, "Retrieve and verify stream");
      asyncTest.done();
    });
  }).catch(function(e) {
    asyncTest.step(function() {
      assert_unreached("Cannot read stream " + e);
      asyncTest.done();
    });
  });
}
