from six import text_type

def main(request, response):
  if "clear-vary-value-override-cookie" in request.GET:
    response.unset_cookie("vary-value-override")
    return "vary cookie cleared"

  set_cookie_vary = request.GET.first("set-vary-value-override-cookie",
                                      default="")
  if set_cookie_vary:
    response.set_cookie("vary-value-override", set_cookie_vary)
    return "vary cookie set"

  # If there is a vary-value-override cookie set, then use its value
  # for the VARY header no matter what the query string is set to.  This
  # override is necessary to test the case when two URLs are identical
  # (including query), but differ by VARY header.
  cookie_vary = request.cookies.get("vary-value-override");
  if cookie_vary:
    response.headers.set("vary", text_type(cookie_vary))
  else:
    # If there is no cookie, then use the query string value, if present.
    query_vary = request.GET.first("vary", default="")
    if query_vary:
      response.headers.set("vary", query_vary)

  return "vary response"
