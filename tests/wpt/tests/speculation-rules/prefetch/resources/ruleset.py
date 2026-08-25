def main(request, response):
    url = request.GET[b"url"].decode("utf-8")
    uuid = request.GET[b"uuid"].decode("utf-8")
    page = request.GET[b"page"].decode("utf-8")
    valid_json = request.GET[b"valid_json"].decode("utf-8")
    empty_json = request.GET[b"empty_json"].decode("utf-8")
    fail_cors = request.GET[b"fail_cors"].decode("utf-8")
    valid_encoding = request.GET[b"valid_encoding"].decode("utf-8")
    redirect = request.GET[b"redirect"].decode("utf-8")
    sec_fetch_dest = request.headers[b"Sec-Fetch-Dest"].decode(
        "utf-8").lower() if b"Sec-Fetch-Dest" in request.headers else None
    content_type = b"application/speculationrules+json" if request.GET[
        b"valid_mime"].decode("utf-8") == "true" else b"application/json"
    status = int(request.GET[b"status"])

    if redirect == "true":
        new_url = request.url.replace("redirect=true",
                                      "redirect=false").encode("utf-8")
        return 301, [(b"Location", new_url),
                     (b'Access-Control-Allow-Origin', b'*')], b""

    encoding = "utf-8" if valid_encoding == "true" else "windows-1250"
    content_type += f'; charset={encoding}'.encode('utf-8')
    strparam = b'\xc3\xb7'.decode('utf-8')

    content = f'''
      {{
        "prefetch": [
          {{
            "source":"list",
            "urls":["{url}?uuid={uuid}&page={page}&str={strparam}"]
          }}
        ]
      }}
  '''
    if empty_json == "true":
        content = ""
    elif valid_json != "true":
        content = "invalid json"
    elif sec_fetch_dest is None:
        content = "Missing Sec-Fetch-Dest"
    elif sec_fetch_dest != "speculationrules":
        content = "Unexpected Sec-Fetch-Dest " + sec_fetch_dest

    headers = [(b"Content-Type", content_type)]
    if fail_cors != "true":
        origin = request.headers[
            b"Origin"] if b"Origin" in request.headers else b'*'
        headers.append((b'Access-Control-Allow-Origin', origin))
    return status, headers, content.encode(encoding)
