# Copyright 2018 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

"""
Created on Mon Mar 26 20:08:25 2018
@author: Pranshu Sinha, Abhay Soni, Aayushi Agrawal

The below program is intended to test rendering mismatches in servo by taking screenshots of rendered html files.
Here is the breakdown of how our code works:
*   A session is started on localhost:7002
*   The randomly generated webpage's (html files) data is sent as JSON to this session
*   Using curl request, we load the html files for this session ID based on the session we just created.
"""

import os
import json
import requests
import start_servo
import time
import base64
import sys
import getopt


def print_help():

    print('\nPlease enter the command as shown below: \n')
    print('python3 ./etc/servo_automation_screenshot.py -p <port>'
          + ' -i /path/to/folder/containing/files -r <resolution>'
          + ' -n num_of_files\n')


def servo_ready_to_accept(url, payload, headers):
    while (True):

        try:
            # Before sending an additional request, we wait for one second each time
            time.sleep(1)
            session_request = requests.post(url, data=payload, headers=headers)
            json_string = session_request.json()
            # On success, we move on to render the files
            break
        except Exception as e:
            time.sleep(5)
            print('Exception: ', e)
    return json_string


def ensure_screenshots_directory_exists():
    if not os.path.exists('screenshots'):
        os.makedirs('screenshots')


def render_html_files(num_of_files, url, file_url, json_string, headers, cwd):
    for x in range(num_of_files):

        json_data = {}
        json_data['url'] = 'file://{0}file{1}.html'.format(file_url, str(x))
        print(json_data['url'])
        json_data = json.dumps(json_data)
        requests.post('{0}/{1}/url'.format(url, json_string['value']['sessionId']), data=json_data, headers=headers)
        screenshot_request = requests.get('{0}/{1}/screenshot'.format(url, json_string['value']['sessionId']))
        image_data_encoded = screenshot_request.json()['value']
        with open("screenshots/output_image_{0}.png".format(str(x)), "wb") as image_file:
            image_file.write(base64.decodebytes(image_data_encoded.encode('utf-8')))
        print("################################")
        print("The screenshot is stored in the location: {0}/screenshots/"
              " with filename: output_image_{1}.png".format(cwd, str(x)))
        print("################################")


def main(argv):  # take inputs from command line by considering the options parameter i.e -h, -p etc.

    # Local Variables
    port = ''
    resolution = ''
    file_url = ''
    num_of_files = ''
    cwd = os.getcwd()
    url = ''
    payload = "{}"
    headers = {'content-type': 'application/json', 'Accept-Charset': 'UTF-8'}
    json_string = ''
    try:
        # input options defined here.
        opts, args = getopt.getopt(argv, "p:i:r:n:", ["port=", "ifile=", "resolution=", "num-files="])
    except getopt.GetoptError:
        # an error is generated if the options provided in commandline are wrong.
        # The help statement is printed on how to input command line arguments.
        print_help()
        sys.exit(2)
    for opt, arg in opts:
        if opt == '-h':  # -h means help. Displays how to input command line arguments
            print_help()
            sys.exit()
        elif opt in ("-p", "--port"):  # store the value provided with the option -p in port variable.
            port = arg
        elif opt in ("-i", "--ifile"):  # store the value provided with the option -i in inputfile variable.
            file_url = arg
        elif opt in ("-r", "--resolution"):  # store the value provided with the option -r in resolution variable.
            resolution = arg
        elif opt in ("-n", "--num-files"):  # store the value provided with the option -n in num_of_files variable.
            num_of_files = arg

    url = 'http://localhost:{0}/session'.format(port)
    num_of_files = int(num_of_files)

    # Starting servo on specified port
    start_servo.start_servo(port, resolution)

    # Waiting until servo is ready to render files
    json_string = servo_ready_to_accept(url, payload, headers)

    # Making sure the screenshots directory exists, if not, create it
    ensure_screenshots_directory_exists()

    # Render each HTML file and take a screenshot
    render_html_files(num_of_files, url, file_url, json_string, headers, cwd)


if __name__ == "__main__":
    if len(sys.argv) < 8:
        print_help()
        sys.exit()
    else:
        main(sys.argv[1:])
