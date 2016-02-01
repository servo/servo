def main(request, response):
    stashed_data = {'count': 0, 'preflight': "0"}
    status = 302
    headers = [("Content-Type", "text/plain"),
               ("Cache-Control", "no-cache"),
               ("Pragma", "no-cache"),
               ("Access-Control-Allow-Origin", "*")]
    token = None

    if "token" in request.GET:
        token = request.GET.first("token")
        data = request.server.stash.take(token)
        if data:
            stashed_data = data

    if request.method == "OPTIONS":
        if "allow_headers" in request.GET:
            headers.append(("Access-Control-Allow-Headers", request.GET['allow_headers']))
        stashed_data['preflight'] = "1"
        #Preflight is not redirected: return 200
        if not "redirect_preflight" in request.GET:
            if token:
              request.server.stash.put(request.GET.first("token"), stashed_data)
            return 200, headers, ""

    if "redirect_status" in request.GET:
        status = int(request.GET['redirect_status'])

    stashed_data['count'] += 1

    #keep url parameters in location
    url_parameters = "?" + "&".join(map(lambda x: x[0][0] + "=" + x[1][0], request.GET.items()))
    #make sure location changes during redirection loop
    url_parameters += "&count=" + str(stashed_data['count'])

    if "location" in request.GET:
        headers.append(("Location", request.GET['location'] + url_parameters))

    if token:
        request.server.stash.put(request.GET.first("token"), stashed_data)
        if "max_count" in request.GET:
            max_count =  int(request.GET['max_count'])
            #stop redirecting and return count
            if stashed_data['count'] > max_count:
                # -1 because the last is not a redirection
                return str(stashed_data['count'] - 1)

    return status, headers, ""
