import os
import time

def main(request, response):
    key = request.GET.first(b"key")

    if request.method == "POST":
        request.server.stash.put(key, True)
        return f"put {key} into stash"

    file_path = os.path.join(request.doc_root, "media", "movie_300.webm")
    with open(file_path, "rb") as f:
        f.seek(0, os.SEEK_END)
        file_size = f.tell()

        f.seek(0, os.SEEK_SET)

        response.add_required_headers = False
        response.writer.write_status(200)
        response.writer.write_header("Content-Type", "video/webm")
        response.writer.write_header("Content-Length", str(file_size))
        response.writer.end_headers()

        # Send a small initial chunk so the browser doesn't buffer enough data
        # to satisfy preload heuristics, which would stop it from requesting more
        # and prevent the stalled event from firing.
        first_size = 4096

        response.writer.write(f.read(first_size))

        # Wait for the key to appear in the stash.
        while True:
            if request.server.stash.take(key) == True:
                break
            time.sleep(0.1)

        # Send the rest of the data.
        response.writer.write(f.read(file_size - first_size))
