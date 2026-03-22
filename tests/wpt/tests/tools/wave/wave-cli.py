# mypy: allow-untyped-defs
# mypy: allow-untyped-calls

import sys
import os
import urllib
import zipfile

START = "start"
DOWNLOAD_REFERENCE_BROWSERS = "download-reference-results"

REFERENCE_BROWSERS = {
    "chrome": {
        "name": "Chromium 73.0.3640.0",
        "url": "https://s3.us-east-2.amazonaws.com/wave-browser-snapshots\
                /wave-reference-browser-results/WMAS+2018\
                /Chromium73-a50c6db0-6a94-11e9-8d1b-e23fc4555885.zip",
    },
    "edge": {
        "name": "Edge 44.17763",
        "url": "https://s3.us-east-2.amazonaws.com/wave-browser-snapshots\
                /wave-reference-browser-results/WMAS+2018\
                /Edge44-b2924d20-6a93-11e9-98b4-a11fb92a6d1c.zip",
    },
    "firefox": {
        "name": "Firefox 64.0",
        "url": "https://s3.us-east-2.amazonaws.com/wave-browser-snapshots\
                /wave-reference-browser-results/WMAS+2018\
                /Firefox64-bb7aafa0-6a92-11e9-8ec2-04f58dad2e4f.zip",
    },
    "webkit": {
        "name": "WebKit r239158",
        "url": "https://s3.us-east-2.amazonaws.com/wave-browser-snapshots\
                /wave-reference-browser-results/WMAS+2018\
                /WebKitr239158-caf823e0-6a92-11e9-b732-3188d0065ebc.zip",
    },
}


def main():
    parameters = get_run_parameters()
    # configuration_file_path = None
    # if ("configuration_file_path" in parameters):
    #    configuration_file_path = parameters["configuration_file_path"]

    if parameters["operation"] == DOWNLOAD_REFERENCE_BROWSERS:
        download_reference_browsers()


def get_run_parameters():
    arguments = sys.argv
    parameters = {}

    operation = arguments[1].lower()

    if operation != START and operation != DOWNLOAD_REFERENCE_BROWSERS:
        raise Exception(f"Unknown operation {operation}")

    parameters["operation"] = operation

    iterator = iter(arguments)
    next(iterator)
    next(iterator)
    for argument in iterator:
        if argument.lower() == "--config":
            path = next(iterator)
            if not path.startswith("/"):
                path = os.path.join(os.getcwd(), path)
            parameters["configuration_file_path"] = path
            continue

        raise Exception(f"Unknown option {argument}")

    if "configuration_file_path" not in parameters:
        configuration_file_path = os.path.join(os.getcwd(), "config.json")
        parameters["configuration_file_path"] = configuration_file_path

    return parameters


def download_file(url, file_path):
    response = urllib.request.urlopen(url)
    data = response.read()
    file = open(file_path, "wb")
    file.write(data)
    file.close()


def printt(text):
    sys.stdout.write(text)
    sys.stdout.flush()


def download_reference_browsers():
    result_directory = os.path.abspath("./results")

    if not os.path.isdir(result_directory):
        os.mkdir(result_directory)

    for id in REFERENCE_BROWSERS:
        browser = REFERENCE_BROWSERS[id]
        browser["zip"] = browser["url"].split("/")[-1]
        printt(f"Downloading {browser['name']} results ...")
        dest_path = os.path.join(result_directory, browser["zip"])
        download_file(browser["url"], dest_path)
        print(" done.")

    for id in REFERENCE_BROWSERS:
        browser = REFERENCE_BROWSERS[id]
        printt(f"Extracting {browser['name']} results ...")
        dest_path = os.path.join(result_directory, browser["name"])
        zip_file = zipfile.ZipFile(dest_path)
        zip_file.extractall(result_directory)
        print(" done.")

    print("Cleaning ...")
    for id in REFERENCE_BROWSERS:
        browser = REFERENCE_BROWSERS[id]
        os.remove(os.path.join(result_directory, browser["zip"]))


main()
