import os
import time

# How long a resume stays in effect. Every request that reaches the stall point
# during this window is released, not just the first one to claim the key.
RESUME_WINDOW = 5

# Waiting is bounded so that a request the UA has already cancelled doesn't park
# its server thread for the rest of the run.
RESUME_TIMEOUT = 30


def main(request, response):
    key = request.GET.first(b"key")
    stash = request.server.stash

    if request.method == "POST":
        stash.put(key, True)

        # Hold the resume open long enough for every request already waiting --
        # and any that arrive just afterwards -- to be released, then take it
        # back out so the value doesn't outlive the test.
        time.sleep(RESUME_WINDOW)
        with stash.lock:
            stash.take(key)

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

        # Wait for the key to appear in the stash. Several requests can be
        # waiting on the same key at once, because the UA may try more than one
        # media player before settling on one that can handle the response.
        deadline = time.monotonic() + RESUME_TIMEOUT
        while True:
            with stash.lock:
                if stash.take(key) == True:
                    # `take` is destructive, so put the value straight back --
                    # under the lock, making the pair atomic -- so that every
                    # request waiting on this key is released, not just whichever
                    # one happened to get here first.
                    stash.put(key, True)
                    break
            if time.monotonic() > deadline:
                return
            time.sleep(0.1)

        # Send the rest of the data.
        response.writer.write(f.read(file_size - first_size))
