""" Handle the sub-resource requests and attach the received client info to
the response.
"""


def main(request, response):
    response.status = 200

    # Echo the received CH headers.
    response.headers.set(
        b"server_received_bitness",
        "true" if b"sec-ch-ua-bitness" in request.headers else "false")
    response.headers.set(
        b"server_received_full_version_list", "true"
        if b"sec-ch-ua-full-version-list" in request.headers else "false")
