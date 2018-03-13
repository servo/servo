def main(request, response):
    return ([("Content-Type", "text/html")], "<script>parent.postMessage(\"" + str(request.POST.first("testinput")) + "\", '*');</script>")
