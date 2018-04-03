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
"""

"""
The below program is intended to test rendering mismatches in servo by taking screenshots of rendered html files.
Here is the breakdown of how our code works:
*   A session is started on localhost:7002
*   The randomly generated webpage's (html files) data is sent as JSON to this session
*   Using curl request, we load the html files for this session ID based on the session we just created.
"""
import os
# import subprocess
import json
import requests
import start_servo
import time
import base64
import sys
import getopt  # for command line interaction
# import socket


port = ''
resolution = ''
url_cl = ''
file_url = ''


def main(argv):  # take inputs from command line by considering the options parameter i.e -h, -p etc.
    try:
        # input options defined here.
        opts, args = getopt.getopt(argv, "hu:p:i:r:", ["url=", "port=", "ifile=", "resolution="])
    except getopt.GetoptError:
        # an error is generated if the options provided in commandline are wrong.
        # The help statement is printed on how to input command line arguments.
        print('python3 etc/servo_automation_screenshot.py -u <url> -p <port> -i <html_file_url> -r <resolution>')
        sys.exit(2)
    for opt, arg in opts:
        if opt == '-h':  # -h means help. Displays how to input command line arguments
            print('python etc/servo_automation_screenshot.py -u <url> -p <port> -i <html_file_url> -r <resolution>')
            sys.exit()
        elif opt in ("-p", "--port"):  # store the value provided with the option -p in port variable.
            global port
            port = arg
        elif opt in ("-u", "--url"):  # store the value provided with the option -u in url_cl variable.
            global url_cl
            url_cl = arg
        elif opt in ("-i", "--ifile"):  # store the value provided with the option -i in inputfile variable.
            global file_url
            file_url = arg
        elif opt in ("-r", "--resolution"):  # store the value provided with the option -i in inputfile variable.
            global resolution
            resolution = arg

# This is to verify the name of the file is correct.
# This is because the name of the file is the first argument passed through the command line.
# After it is true it takes the command line arguments and processes it using the function main()
if __name__ == "__main__":
    if len(sys.argv) < 2:
        print('Argument input is required in the format')
        print('python3 etc/servo_automation_screenshot.py -u <url> -p <port> -i <html_file_url> -r <resolution>')
        sys.exit()
    else:
        main(sys.argv[1:])


# The below function is used to start servo on localhost:7002
process_servo = start_servo.start_servo(port, resolution)

# Since servo takes time to load and to get the session ID for subsequent steps, we added a sleep for 60 seconds
time.sleep(60)
# We are waiting for reponse and we will edit as soon as we get the response.

# sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
# connected = False
# while not connected:
#    try:
#        sock.connect(('localhost',7002))
#        connected = True
#    except Exception as e:
#        pass #Do nothing, just try again
#        #print("Retrying Servo")
#
# print(connected)

# The program assumes that the html files to be rendered in servo are present in the
# same current working directory in the format "file#.html"
cwd = os.getcwd()
url = 'http://' + url_cl + ':' + port + '/session'
payload = "{}"
headers = {'content-type': 'application/json', 'Accept-Charset': 'UTF-8'}
session_request = requests.post(url, data=payload, headers=headers)
json_string = session_request.json()
# Currently, we have generated 5 random html pages, hence the loop runs for 5 iterations
for x in range(5):
    json_data = {}
    json_data['url'] = 'file://' + file_url + 'file' + str(x) + '.html'
    json_data = json.dumps(json_data)
    payload2 = json_data
    url_request = requests.post(url + json_string['value']['sessionId'] + '/url', data=payload2, headers=headers)
    screenshot_request = requests.get(url + json_string['value']['sessionId'] + '/screenshot')
    image_data_encoded = screenshot_request.json()['value']
    with open("screenshots/output_image_" + str(x) + ".png", "wb") as image_file:
        image_file.write(base64.decodebytes(image_data_encoded.encode('utf-8')))
    print("################################")
    print("The screenshot is stored in the location: " + cwd +
          "/screenshots/ with filename: output_image_" + str(x) + ".png")
print("################################")
process_servo.stdin.close()
