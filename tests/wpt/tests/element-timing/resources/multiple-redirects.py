from wptserve.utils import isomorphic_encode

def main(request, response):
    """Handler that causes multiple redirections.

    Mandatory parameters:
    redirect_count - A number which is at least 1 (number of redirects).
    final_resource - The location of the last redirect.

    For each number i between 1 and |redirect_count| we have the following optional parameters:
    tao{{i}} - The Timing-Allow-Origin header of the ith response. Default is no header.
    origin{{i}} - The origin of the ith redirect (i+1 response). Default is location.origin.
    Note that the origin of the initial request cannot be controlled here
    and the Timing-Allow-Origin header of the final response cannot be controlled here.

    Example: redirect_count=2&final_resource=miau.png&tao1=*

    Note: |step| is used internally to track the current redirect number.
    """
    step = 1
    if b"step" in request.GET:
        try:
            step = int(request.GET.first(b"step"))
        except ValueError:
            pass

    redirect_count = int(request.GET.first(b"redirect_count"))
    final_resource = request.GET.first(b"final_resource")

    tao_value = None
    tao = b"tao" + isomorphic_encode(str(step))
    if tao in request.GET:
        tao_value = request.GET.first(tao)

    redirect_url = b""
    origin = b"origin" + isomorphic_encode(str(step))
    if origin in request.GET:
        redirect_url = request.GET.first(origin)

    if step == redirect_count:
        redirect_url += final_resource
    else:
        redirect_url += b"/element-timing/resources/multiple-redirects.py?"
        redirect_url += b"redirect_count=" + isomorphic_encode(str(redirect_count))
        redirect_url += b"&final_resource=" + final_resource
        for i in range(1, redirect_count + 1):
            tao = b"tao" + isomorphic_encode(str(i))
            if tao in request.GET:
                redirect_url += b"&" + tao + b"=" + request.GET.first(tao)
            origin = b"origin" + isomorphic_encode(str(i))
            if origin in request.GET:
                redirect_url += b"&" + origin + b"=" + request.GET.first(origin)
        redirect_url += b"&step=" + isomorphic_encode(str(step + 1))

    if tao_value:
        response.headers.set(b"timing-allow-origin", tao_value)

    response.status = 302
    response.headers.set(b"Location", redirect_url)
