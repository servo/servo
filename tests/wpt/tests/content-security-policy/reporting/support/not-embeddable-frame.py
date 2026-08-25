def main(request, response):
    headers = []
    if request.GET.first(b'xFrameOptions', None):
        headers.append((b'X-Frame-Options', request.GET[b'xFrameOptions']))

    csp_header = b'Content-Security-Policy-Report-Only' \
        if request.GET.first(b'reportOnly', None) == b'true' else b'Content-Security-Policy'
    report_uri_base = request.GET.first(b'reportUriBase', b'')
    headers.append((csp_header, b"frame-ancestors 'none'; report-uri " + report_uri_base + b"/reporting/resources/report.py?op=put&reportID=" + request.GET[b'reportID']))

    return headers, b'{}'
