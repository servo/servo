if (this.document === undefined) {
  importScripts("/resources/testharness.js");
  importScripts("../resources/utils.js");
}

function integrity(desc, url, integrity, shouldPass) {
   if (shouldPass) {
    promise_test(function(test) {
      return fetch(url, {'integrity': integrity}).then(function(resp) {
        assert_equals(resp.status, 200, "Response's status is 200");
      });
    }, desc);
  } else {
    promise_test(function(test) {
      return promise_rejects(test, new TypeError(), fetch(url, {'integrity': integrity}));
    }, desc);
  }
}

var topSha256 = "sha256-KHIDZcXnR2oBHk9DrAA+5fFiR6JjudYjqoXtMR1zvzk=";
var topSha384 = "sha384-MgZYnnAzPM/MjhqfOIMfQK5qcFvGZsGLzx4Phd7/A8fHTqqLqXqKo8cNzY3xEPTL";
var topSha512 = "sha512-D6yns0qxG0E7+TwkevZ4Jt5t7Iy3ugmAajG/dlf6Pado1JqTyneKXICDiqFIkLMRExgtvg8PlxbKTkYfRejSOg==";
var invalidSha256 = "sha256-dKUcPOn/AlUjWIwcHeHNqYXPlvyGiq+2dWOdFcE+24I=";
var invalidSha512 = "sha512-oUceBRNxPxnY60g/VtPCj2syT4wo4EZh2CgYdWy9veW8+OsReTXoh7dizMGZafvx9+QhMS39L/gIkxnPIn41Zg==";

var url = "../resources/top.txt";
var corsUrl = "http://www1.{{host}}:{{ports[http][0]}}" + dirname(location.pathname) + RESOURCES_DIR + "top.txt";
/* Enable CORS*/
corsUrl += "?pipe=header(Access-Control-Allow-Origin,*)";

integrity("Empty string integrity", url, "", true);
integrity("SHA-256 integrity", url, topSha256, true);
integrity("SHA-384 integrity", url, topSha384, true);
integrity("SHA-512 integrity", url, topSha512, true);
integrity("Invalid integrity", url, invalidSha256, false);
integrity("Multiple integrities: valid stronger than invalid", url, invalidSha256 + " " + topSha384, true);
integrity("Multiple integrities: invalid stronger than valid", url, invalidSha512 + " " + topSha384, false);
integrity("Multiple integrities: invalid as strong as valid", url, invalidSha512 + " " + topSha512, true);
integrity("Multiple integrities: both are valid", url,  topSha384 + " " + topSha512, true);
integrity("Multiple integrities: both are invalid", url, invalidSha256 + " " + invalidSha512, false);
integrity("CORS empty integrity", corsUrl, "", true);
integrity("CORS SHA-512 integrity", corsUrl, topSha512, true);
integrity("CORS invalid integrity", corsUrl, invalidSha512, false);

done();
