def main(request, response):
    """Handler that causes multiple redirections. Redirect chain is as follows:
        1. Initial URL containing multi-redirect.py
        2. Redirect to cross-origin URL
        3. Redirect to same-origin URL
        4. Final URL containing the final same-origin resource.
    Mandatory parameters:
    page_origin - The page origin, used for redirection and to set TAO. This is a mandatory parameter.
    cross_origin - The cross origin used to make this a cross-origin redirect. This is a mandatory parameter.
    final_resource - Path of the final resource, without origin. This is a mandatory parameter.
    Optional parameters:
    tao_steps - Number of redirects for which the TAO header will be present (a number 0 - 3 makes the most sense). Default value is 0.
    tao_value - The value of the TAO header, when present. Default value is "*".
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
    final_resource = request.GET.first("final_resource")

    tao_value = "*";
    if "tao_value" in request.GET:
        tao_value = request.GET.first("tao_value")
    tao_steps = 0
    if "tao_steps" in request.GET:
        tao_steps = int(request.GET.first("tao_steps"))

    next_tao_steps = tao_steps - 1
    redirect_url_path = "/resource-timing/resources/multi_redirect.py?"
    redirect_url_path += "page_origin=" + page_origin
    redirect_url_path += "&cross_origin=" + cross_origin
    redirect_url_path += "&final_resource=" + final_resource
    redirect_url_path += "&tao_value=" + tao_value
    redirect_url_path += "&tao_steps=" + str(next_tao_steps)
    redirect_url_path += "&step="
    if tao_steps > 0:
        response.headers.set("timing-allow-origin", tao_value)

    if step == 1:
        # On the first request, redirect to a cross origin URL
        redirect_url = cross_origin + redirect_url_path + "2"
    elif step == 2:
        # On the second request, redirect to a same origin URL
        redirect_url = page_origin + redirect_url_path + "3"
    else:
        # On the third request, redirect to a static response
        redirect_url = page_origin + final_resource

    response.status = 302
    response.headers.set("Location", redirect_url)
