def main(request, response):
    response.headers.set("Content-Type", request.GET.first("type"));
    response.content = "<meta charset=utf-8>\n<script>document.write(document.characterSet)</script>"
