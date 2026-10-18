def main(request, response):
    csp = request.GET.first(b"policy")
    headers = [
      (b"Content-Type", b"text/html"),
      (b"Content-Security-Policy", csp)
    ]

    body = f"""<!DOCTYPE html>
        <html>
        <head>
          <title>CSP</title>
        </head>
        <body>
          <p>{csp}</p>
          <script>
            // Attach the handlers before setting `src`, so a cached or
            // quickly-blocked image can't fire load/error before they exist.
            let img = document.createElement("img");
            img.onload = img.onerror = (event) => {{
              window.parent.postMessage(event.type, '*');
            }};
            img.src = "/content-security-policy/support/pass.png";
            document.body.appendChild(img);
          </script>
        </body>
        </html>
    """
    return (headers, body)
