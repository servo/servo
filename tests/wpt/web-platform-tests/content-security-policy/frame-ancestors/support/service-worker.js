self.onfetch = e => {
  e.respondWith(function() {
    return new Promise((resolve) => {
      var headers = new Headers;
      headers.append("Content-Security-Policy", "frame-ancestors 'none'");
      var response = new Response("", { "headers" : headers, "status": 200, "statusText" : "OK" });
      resolve(response);
    });
  }());
};
