if (this.document === undefined) {
  importScripts("/resources/testharness.js");
  importScripts("../resources/utils.js");
}

function corsFilter(corsUrl, headerName, headerValue, isFiltered) {
  var url = corsUrl + "?pipe=header(" + headerName + "," + encodeURIComponent(headerValue) +")|header(Access-Control-Allow-Origin,*)";
  promise_test(function(test) {
    return fetch(url).then(function(resp) {
      assert_equals(resp.status, 200, "Fetch success with code 200");
      assert_equals(resp.type , "cors", "CORS fetch's response has cors type");
      if (!isFiltered) {
        assert_equals(resp.headers.get(headerName), headerValue,
          headerName + " header should be included in response with value: " + headerValue);
      } else {
        assert_false(resp.headers.has(headerName), "UA should exclude " + headerName + " header from response");
      }
      test.done();
    });
  }, "CORS filter on " + headerName + " header");
}

function corsExposeFilter(corsUrl, headerName, headerValue, isForbidden) {
  var url = corsUrl + "?pipe=header(" + headerName + "," + encodeURIComponent(headerValue) +")|" +
                            "header(Access-Control-Allow-Origin,*)" +
                            "header(Access-Control-Expose-Headers," + headerName + ")";

  promise_test(function(test) {
    return fetch(url).then(function(resp) {
      assert_equals(resp.status, 200, "Fetch success with code 200");
      assert_equals(resp.type , "cors", "CORS fetch's response has cors type");
      if (!isForbidden) {
        assert_equals(resp.headers.get(headerName), headerValue,
          headerName + " header should be included in response with value: " + headerValue);
      } else {
        assert_false(resp.headers.has(headerName), "UA should exclude " + headerName + " header from response");
      }
      test.done();
    });
  }, "CORS filter on " + headerName + " header, header is exposed");
}

var url = "http://{{host}}:{{ports[http][1]}}" + dirname(location.pathname) + RESOURCES_DIR + "top.txt";

corsFilter(url, "Cache-Control", "no-cache", false);
corsFilter(url, "Content-Language", "fr", false);
corsFilter(url, "Content-Type", "text/html", false);
corsFilter(url, "Expires","04 May 1988 22:22:22 GMT" , false);
corsFilter(url, "Last-Modified", "04 May 1988 22:22:22 GMT", false);
corsFilter(url, "Pragma", "no-cache", false);

corsFilter(url, "Age", "27", true);
corsFilter(url, "Server", "wptServe" , true);
corsFilter(url, "Warning", "Mind the gap" , true);
corsFilter(url, "Content-Length", "0" , true);
corsFilter(url, "Set-Cookie", "name=value" , true);
corsFilter(url, "Set-Cookie2", "name=value" , true);

corsExposeFilter(url, "Age", "27", false);
corsExposeFilter(url, "Server", "wptServe" , false);
corsExposeFilter(url, "Warning", "Mind the gap" , false);
corsExposeFilter(url, "Content-Length", "0" , false);
corsExposeFilter(url, "Set-Cookie", "name=value" , true);
corsExposeFilter(url, "Set-Cookie2", "name=value" , true);

done();
