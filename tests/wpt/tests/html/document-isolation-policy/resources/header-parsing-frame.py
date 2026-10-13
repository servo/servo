def main(request, response):
    headers = [(b'Content-Type', b'text/html')]

    for value in request.GET.get_list(b'value'):
        headers.append((b'Document-Isolation-Policy', value))

    body = u'''<!doctype html>
<script>
const params = new URL(location).searchParams;
fetch(params.get("url"), { mode: "no-cors" })
  .then(() => "allowed", () => "blocked")
  .then(result => parent.postMessage({ id: params.get("id"), result }, "*"));
</script>'''
    return (200, headers, body)
