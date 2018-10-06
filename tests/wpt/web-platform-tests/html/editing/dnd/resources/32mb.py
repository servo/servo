thirty_two_megabytes = 32 * 1024 * 1024
chunk = 'ab' * 512 * 512
chunk_length = len(chunk)

def main(request, response):
    def content():
        bytes_sent = 0
        while bytes_sent < thirty_two_megabytes:
            yield chunk
            bytes_sent += chunk_length

    return [("Content-Type", "text/plain")], content()
