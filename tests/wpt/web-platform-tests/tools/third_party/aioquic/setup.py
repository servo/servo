import os.path
import sys

import setuptools

root_dir = os.path.abspath(os.path.dirname(__file__))

about = {}
about_file = os.path.join(root_dir, "src", "aioquic", "about.py")
with open(about_file, encoding="utf-8") as fp:
    exec(fp.read(), about)

readme_file = os.path.join(root_dir, "README.rst")
with open(readme_file, encoding="utf-8") as f:
    long_description = f.read()

if sys.platform == "win32":
    extra_compile_args = []
    libraries = ["libcrypto", "advapi32", "crypt32", "gdi32", "user32", "ws2_32"]
else:
    extra_compile_args = ["-std=c99"]
    libraries = ["crypto"]

setuptools.setup(
    name=about["__title__"],
    version=about["__version__"],
    description=about["__summary__"],
    long_description=long_description,
    url=about["__uri__"],
    author=about["__author__"],
    author_email=about["__email__"],
    license=about["__license__"],
    include_package_data=True,
    classifiers=[
        "Development Status :: 4 - Beta",
        "Environment :: Web Environment",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: BSD License",
        "Operating System :: OS Independent",
        "Programming Language :: Python",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.6",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Topic :: Internet :: WWW/HTTP",
    ],
    ext_modules=[
        setuptools.Extension(
            "aioquic._buffer",
            extra_compile_args=extra_compile_args,
            sources=["src/aioquic/_buffer.c"],
        ),
        setuptools.Extension(
            "aioquic._crypto",
            extra_compile_args=extra_compile_args,
            libraries=libraries,
            sources=["src/aioquic/_crypto.c"],
        ),
    ],
    package_dir={"": "src"},
    package_data={"aioquic": ["py.typed", "_buffer.pyi", "_crypto.pyi"]},
    packages=["aioquic", "aioquic.asyncio", "aioquic.h0", "aioquic.h3", "aioquic.quic"],
    install_requires=[
        "certifi",
        "cryptography >= 2.5",
        'dataclasses; python_version < "3.7"',
        "pylsqpack >= 0.3.3, < 0.4.0",
    ],
)
