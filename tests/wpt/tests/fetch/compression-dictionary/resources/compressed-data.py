def main(request, response):
    response.headers.set(b"Access-Control-Allow-Origin", b"*")
    response.headers.set(b"Content-Type", b"text/plain")
    response.headers.set(
        b"Content-Dictionary",
        b":U5abz16WDg7b8KS93msLPpOB4Vbef1uRzoORYkJw9BY=:")

    # `br_d_data` and `zstd_d_data` are generated using the following commands:
    #
    # $ echo "This is a test dictionary." > /tmp/dict
    # $ echo -n "This is compressed test data using a test dictionary" \
    #    > /tmp/data
    # $ brotli -o /tmp/out.brd -D /tmp/dict /tmp/data
    # $ xxd -p /tmp/out.brd | tr -d '\n' | sed 's/\(..\)/\\x\1/g'
    br_d_data = b"\xa1\x98\x01\x80\x62\xa4\x4c\x1d\xdf\x12\x84\x8c\xae\xc2\xca\x60\x22\x07\x6e\x81\x05\x14\xc9\xb7\xc3\x44\x8e\xbc\x16\xe0\x15\x0e\xec\xc1\xee\x34\x33\x3e\x0d"
    # $ zstd -o /tmp/out.zstdd -D /tmp/dict /tmp/data
    # $ xxd -p /tmp/out.zstdd | tr -d '\n' | sed 's/\(..\)/\\x\1/g'
    zstd_d_data = b"\x28\xb5\x2f\xfd\x24\x34\xf5\x00\x00\x98\x63\x6f\x6d\x70\x72\x65\x73\x73\x65\x64\x61\x74\x61\x20\x75\x73\x69\x6e\x67\x03\x00\x59\xf9\x73\x54\x46\x27\x26\x10\x9e\x99\xf2\xbc"

    if b'content_encoding' in request.GET:
        content_encoding = request.GET.first(b"content_encoding")
        response.headers.set(b"Content-Encoding", content_encoding)
        if content_encoding == b"br-d":
            # Send the pre compressed file
            response.content = br_d_data
        if content_encoding == b"zstd-d":
            # Send the pre compressed file
            response.content = zstd_d_data
