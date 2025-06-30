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
          <img src="/content-security-policy/support/pass.png" />
          <script>
            let img = document.querySelector("img");
            img.onload = img.onerror = (event) => {{
              window.parent.postMessage(event.type, '*');
            }};
          </script>
        </body>
        </html>
    """
    return (headers, body)
