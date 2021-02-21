import re
from pathlib import Path

from setuptools import setup, find_packages


def find_version():
    version_file = (
        Path(__file__)
        .parent.joinpath("pytest_asyncio", "__init__.py")
        .read_text()
    )
    version_match = re.search(
        r"^__version__ = ['\"]([^'\"]*)['\"]", version_file, re.M
    )
    if version_match:
        return version_match.group(1)

    raise RuntimeError("Unable to find version string.")


setup(
    name="pytest-asyncio",
    version=find_version(),
    packages=find_packages(),
    url="https://github.com/pytest-dev/pytest-asyncio",
    license="Apache 2.0",
    author="Tin Tvrtković",
    author_email="tinchester@gmail.com",
    description="Pytest support for asyncio.",
    long_description=Path(__file__).parent.joinpath("README.rst").read_text(),
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: Apache Software License",
        "Programming Language :: Python :: 3.5",
        "Programming Language :: Python :: 3.6",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Topic :: Software Development :: Testing",
        "Framework :: Pytest",
    ],
    python_requires=">= 3.5",
    install_requires=["pytest >= 5.4.0"],
    extras_require={
        ':python_version == "3.5"': "async_generator >= 1.3",
        "testing": [
            "coverage",
            "async_generator >= 1.3",
            "hypothesis >= 5.7.1",
        ],
    },
    entry_points={"pytest11": ["asyncio = pytest_asyncio.plugin"]},
)
