def main(request, response):
    """Handler that causes multiple redirections.
    The request has two mandatory and one optional query parameters:
    page_origin - The page origin, used for redirection and to set TAO. This is a mandatory parameter.
    cross_origin - The cross origin used to make this a cross-origin redirect. This is a mandatory parameter.
    timing_allow - Whether TAO should be set or not in the redirect chain. This is an optional parameter. Default: not set.
    Note that |step| is a parameter used internally for the multi-redirect. It's the step we're at in the redirect chain.
    """
    step = 1
    if "step" in request.GET:
        try:
            step = int(request.GET.first("step"))
        except ValueError:
            pass

    origin = request.url_parts.scheme + "://" + request.url_parts.hostname + ":" + str(request.url_parts.port)
    page_origin = request.GET.first("page_origin")
    cross_origin = request.GET.first("cross_origin")
    timing_allow = "0"
    if "timing_allow" in request.GET:
        timing_allow = request.GET.first("timing_allow")

    redirect_url_path = "/resource-timing/resources/multi_redirect.py?"
    redirect_url_path += "page_origin=" + page_origin
    redirect_url_path += "&cross_origin=" + cross_origin
    redirect_url_path += "&timing_allow=" + timing_allow
    redirect_url_path += "&step="

    if step == 1:
        # On the first request, redirect to a cross origin URL
        redirect_url = cross_origin + redirect_url_path + "2"
        if timing_allow != "0" and origin != page_origin:
            response.headers.set("timing-allow-origin", page_origin)
    elif step == 2:
        # On the second request, redirect to a same origin URL
        redirect_url = page_origin + redirect_url_path + "3"
        if timing_allow != "0":
            response.headers.set("timing-allow-origin", page_origin)
    else:
        # On the third request, redirect to a static response
        redirect_url = page_origin + "/resource-timing/resources/blank_page_green.htm"

    response.status = 302
    response.headers.set("Location", redirect_url)
