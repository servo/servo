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
    if "step" in request.GET:
        try:
            step = int(request.GET.first("step"))
        except ValueError:
            pass

    redirect_count = int(request.GET.first("redirect_count"))
    final_resource = request.GET.first("final_resource")

    tao_value = None
    tao = "tao" + str(step)
    if tao in request.GET:
        tao_value = request.GET.first(tao)

    redirect_url = ""
    origin = "origin" + str(step)
    if origin in request.GET:
        redirect_url = request.GET.first(origin)

    if step == redirect_count:
        redirect_url += final_resource
    else:
        redirect_url += "/element-timing/resources/multiple-redirects.py?"
        redirect_url += "redirect_count=" + str(redirect_count)
        redirect_url += "&final_resource=" + final_resource
        for i in range(1, redirect_count + 1):
            tao = "tao" + str(i)
            if tao in request.GET:
                redirect_url += "&" + tao + "=" + request.GET.first(tao)
            origin = "origin" + str(i)
            if origin in request.GET:
                redirect_url += "&" + origin + "=" + request.GET.first(origin)
        redirect_url += "&step=" + str(step + 1)

    if tao_value:
        response.headers.set("timing-allow-origin", tao_value)

    response.status = 302
    response.headers.set("Location", redirect_url)
