import subprocess
import os

here = os.path.dirname(__file__)


def build(*args, **kwargs):
    subprocess.check_call(["make", "html"], cwd=here)
