def main(request, response):
    response.headers.set(b"Access-Control-Allow-Origin", b"*")
    response.headers.set(b"Content-Type", b"text/plain")

    if b'cacheable' in request.GET:
        response.headers.set(b"Cache-Control", b"max-age=3600")

    # `dcb_data` and `dcz_data` are generated using the following commands:
    #
    # $ echo "This is a test dictionary." > /tmp/dict
    # $ echo -n "This is compressed test data using a test dictionary" \
    #    > /tmp/data
    #
    # $ echo -en '\xffDCB' > /tmp/out.dcb
    # $ openssl dgst -sha256 -binary /tmp/dict >> /tmp/out.dcb
    # $ brotli --stdout -D /tmp/dict /tmp/data >> /tmp/out.dcb
    # $ xxd -p /tmp/out.dcb | tr -d '\n' | sed 's/\(..\)/\\x\1/g'
    dcb_data = b"\xff\x44\x43\x42\x53\x96\x9b\xcf\x5e\x96\x0e\x0e\xdb\xf0\xa4\xbd\xde\x6b\x0b\x3e\x93\x81\xe1\x56\xde\x7f\x5b\x91\xce\x83\x91\x62\x42\x70\xf4\x16\xa1\x98\x01\x80\x62\xa4\x4c\x1d\xdf\x12\x84\x8c\xae\xc2\xca\x60\x22\x07\x6e\x81\x05\x14\xc9\xb7\xc3\x44\x8e\xbc\x16\xe0\x15\x0e\xec\xc1\xee\x34\x33\x3e\x0d"
    # $ echo -en '\x5e\x2a\x4d\x18\x20\x00\x00\x00' > /tmp/out.dcz
    # $ openssl dgst -sha256 -binary /tmp/dict >> /tmp/out.dcz
    # $ zstd -D /tmp/dict -f -o /tmp/tmp.zstd /tmp/data
    # $ cat /tmp/tmp.zstd >> /tmp/out.dcz
    # $ xxd -p /tmp/out.dcz | tr -d '\n' | sed 's/\(..\)/\\x\1/g'
    dcz_data = b"\x5e\x2a\x4d\x18\x20\x00\x00\x00\x53\x96\x9b\xcf\x5e\x96\x0e\x0e\xdb\xf0\xa4\xbd\xde\x6b\x0b\x3e\x93\x81\xe1\x56\xde\x7f\x5b\x91\xce\x83\x91\x62\x42\x70\xf4\x16\x28\xb5\x2f\xfd\x24\x34\xf5\x00\x00\x98\x63\x6f\x6d\x70\x72\x65\x73\x73\x65\x64\x61\x74\x61\x20\x75\x73\x69\x6e\x67\x03\x00\x59\xf9\x73\x54\x46\x27\x26\x10\x9e\x99\xf2\xbc"

    # `large_dcb_data` and `large_dcz_data` are generated the same way as
    # above but with a larger input (348 bytes) that compresses well with the
    # dictionary, so the encoded size is smaller than the decoded size:
    #
    # $ echo "This is a test dictionary." > /tmp/dict
    # $ python3 -c "import sys; sys.stdout.write(
    #     'This is a test dictionary. ' * 10 +
    #     'This is additional test data that also references the test '
    #     'dictionary content.')" > /tmp/large_data
    # $ echo -en '\xffDCB' > /tmp/out.dcb
    # $ openssl dgst -sha256 -binary /tmp/dict >> /tmp/out.dcb
    # $ brotli --stdout -D /tmp/dict /tmp/large_data >> /tmp/out.dcb
    # $ xxd -p /tmp/out.dcb | tr -d '\n' | sed 's/\(..\)/\\x\1/g'
    large_dcb_data = b"\xff\x44\x43\x42\x53\x96\x9b\xcf\x5e\x96\x0e\x0e\xdb\xf0\xa4\xbd\xde\x6b\x0b\x3e\x93\x81\xe1\x56\xde\x7f\x5b\x91\xce\x83\x91\x62\x42\x70\xf4\x16\xa1\xd8\x0a\x00\x2f\xea\xb6\x54\x17\xd2\x63\x1f\x3a\x79\x10\x85\x6f\xb0\x01\x07\x6e\x41\x4a\xc3\x42\xb4\xe9\xb0\x01\x17\x71\x94\x7c\xe9\xaf\x81\xdc\xde\xc0\xc8\xc1\x66\x88\x34\x03\x5e\x7f\x8c\x8e\x46\x6f\xc9\x48\xab\xc8\x71\x2c"
    # $ echo -en '\x5e\x2a\x4d\x18\x20\x00\x00\x00' > /tmp/out.dcz
    # $ openssl dgst -sha256 -binary /tmp/dict >> /tmp/out.dcz
    # $ zstd -D /tmp/dict -f -o /tmp/tmp.zstd /tmp/large_data
    # $ cat /tmp/tmp.zstd >> /tmp/out.dcz
    # $ xxd -p /tmp/out.dcz | tr -d '\n' | sed 's/\(..\)/\\x\1/g'
    large_dcz_data = b"\x5e\x2a\x4d\x18\x20\x00\x00\x00\x53\x96\x9b\xcf\x5e\x96\x0e\x0e\xdb\xf0\xa4\xbd\xde\x6b\x0b\x3e\x93\x81\xe1\x56\xde\x7f\x5b\x91\xce\x83\x91\x62\x42\x70\xf4\x16\x28\xb5\x2f\xfd\x64\x5c\x00\xfd\x01\x00\xf4\x02\x20\x64\x64\x69\x74\x69\x6f\x6e\x61\x6c\x61\x74\x61\x20\x74\x68\x61\x74\x20\x61\x6c\x73\x6f\x20\x72\x65\x66\x65\x72\x65\x6e\x63\x65\x73\x20\x74\x68\x65\x20\x63\x6f\x6e\x74\x65\x6e\x74\x2e\x04\x00\x60\x2d\x72\x35\x2b\xbb\x3c\xa0\xce\xed\x19\x04\x0c\x4b\x9e\x2f"

    use_large = b'large' in request.GET

    if b'content_encoding' in request.GET:
        content_encoding = request.GET.first(b"content_encoding")
        response.headers.set(b"Content-Encoding", content_encoding)
        if content_encoding == b"dcb":
            # Send the pre compressed file
            response.content = large_dcb_data if use_large else dcb_data
        if content_encoding == b"dcz":
            # Send the pre compressed file
            response.content = large_dcz_data if use_large else dcz_data





