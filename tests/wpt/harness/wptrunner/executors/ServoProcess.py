import subprocess


class ServoProcess(object):
    def __init__(self):
        self.path = "path/to/servo"
        self.proc = None

    def __enter__(self):
        self.proc = subprocess.Popen(["./mach run --webdriver 7000 tests/html/about-mozilla.html"], shell=True)

    def __exit__(self, *args, **kwargs):
        self.proc.kill()
