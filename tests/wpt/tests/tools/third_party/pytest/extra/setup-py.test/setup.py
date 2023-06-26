import sys
from distutils.core import setup

if __name__ == "__main__":
    if "sdist" not in sys.argv[1:]:
        raise ValueError("please use 'pytest' pypi package instead of 'py.test'")
    setup(
        name="py.test",
        version="0.0",
        description="please use 'pytest' for installation",
    )
