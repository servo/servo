import time
import importlib
compressedData = importlib.import_module("fetch.compression-dictionary.resources.compressed-data")

def handle_headers(frame, request, response):
    (headers, content) = compressedData.headers_and_content(request)
    response.status = 200
    response.headers = headers
    response.write_status_headers()

def main(request, response):
    (headers, content) = compressedData.headers_and_content(request)

    if b'chunked_response' not in request.GET:
        response.writer.write_data(item=content, last=True)
    else:
        chunk_count = 4
        content_size = len(content)
        chunk_size = content_size // chunk_count
        for i in range(0, content_size, chunk_size):
            chunk = content[i:(i + chunk_size)]
            last_chunk = i + chunk_size >= content_size
            time.sleep(.1)
            response.writer.write_data(item=chunk, last=last_chunk)
