def main(request, response):
  """Send a response with the origin policy indicated by the ?policy= argument.

     Won't send a policy when the browser doesn't indicate support.
     The response tests whether inline script and eval are allowed, and will
     send a corresponding message to the parent frame.
     For easier debugging, we'll also show the results in-page.
  """
  origin_policy_header = "Sec-Origin-Policy"
  request_policy = request.headers.get(origin_policy_header)
  response_policy = request.GET.first("policy", default="")

  if request_policy and response_policy:
    response.headers.set(origin_policy_header, "policy=%s" % response_policy)
    response.headers.set("Vary", "sec-origin-policy")

  response.headers.set("Content-Type", "text/html");
  return """
    <html>
    <head>
     <title>Page with an Origin Policy</title>
    </head>
    <body>
    <script nonce=test>
      let inlineAllowed = false;
      let evalAllowed = false;
      try { eval('evalAllowed = true;'); } catch (e) {};
    </script>
    <script>
      inlineAllowed = true;
    </script>

    <p>Reveal whether CSP with "unsafe-inline" or "unsafe-eval" is present:</p>
    <ul>
      <li>inline script allowed: <span id=inline_allowed></span></li>
      <li>eval allowed: <span id=eval_allowed></span></li>
    </ul>

    <script nonce=test>
      const result = {
        "inline_allowed": inlineAllowed,
        "eval_allowed": evalAllowed,
      };

      // Mirror content into the page for easy debugging:
      const styles = {
        true: "font-weight: bold; color: green;",
        false: "font-weight: bold; color: red",
      }
      for (const [key, value] of Object.entries(result)) {
        let element = document.getElementById(key);
        element.textContent = value.toString();
        element.style = styles[value];
      }

      // Send result to parent frame for evaluation.
      if (window.parent) {
        window.parent.postMessage(result, "*");
      }
    </script>
    </body>
    </html>
  """

