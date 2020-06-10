from wptserve.utils import isomorphic_encode

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
    if b"step" in request.GET:
        try:
            step = int(request.GET.first(b"step"))
        except ValueError:
            pass

    page_origin = request.GET.first(b"page_origin")
    cross_origin = request.GET.first(b"cross_origin")
    final_resource = request.GET.first(b"final_resource")

    tao_value = b"*"
    if b"tao_value" in request.GET:
        tao_value = request.GET.first(b"tao_value")
    tao_steps = 0
    if b"tao_steps" in request.GET:
        tao_steps = int(request.GET.first(b"tao_steps"))

    next_tao_steps = tao_steps - 1
    redirect_url_path = b"/resource-timing/resources/multi_redirect.py?"
    redirect_url_path += b"page_origin=" + page_origin
    redirect_url_path += b"&cross_origin=" + cross_origin
    redirect_url_path += b"&final_resource=" + final_resource
    redirect_url_path += b"&tao_value=" + tao_value
    redirect_url_path += b"&tao_steps=" + isomorphic_encode(str(next_tao_steps))
    redirect_url_path += b"&step="
    if tao_steps > 0:
        response.headers.set(b"timing-allow-origin", tao_value)

    if step == 1:
        # On the first request, redirect to a cross origin URL
        redirect_url = cross_origin + redirect_url_path + b"2"
    elif step == 2:
        # On the second request, redirect to a same origin URL
        redirect_url = page_origin + redirect_url_path + b"3"
    else:
        # On the third request, redirect to a static response
        redirect_url = page_origin + final_resource

    response.status = 302
    response.headers.set(b"Location", redirect_url)
