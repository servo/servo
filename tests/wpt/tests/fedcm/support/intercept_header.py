def main(request, response):
  port = request.server.config.ports["https"][0]
  hostname = request.url_parts.hostname
  base_url = f"https://{hostname}:{port}".encode('utf-8')

  header_value = b"client_id=\"1234\", params=\"redirect=post\", config_url=\"%s/fedcm/support/manifest_with_continue_on.json\"" % (base_url)
  response.headers.set(b"fedcm-intercept-navigation", header_value)

  return "Sent header: %s" % (header_value)
