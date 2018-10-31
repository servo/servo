#!/usr/bin/python3

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import json
import sys
import urllib.request

import tc


SERVO_PROJECT_ID = "e3d0d8be-9e4c-4d39-90af-38660eb70544"
PACKET_AUTH_TOKEN = None


def main():
    tc.check()
    global PACKET_AUTH_TOKEN
    PACKET_AUTH_TOKEN = tc.packet_auth_token()
    response = api_request("/projects/%s/devices?per_page=1000" % SERVO_PROJECT_ID)
    for device in response["devices"]:
        print(device["id"])
        print("    Hostname:\t" + device["hostname"])
        print("    Plan:\t" + device["plan"]["name"])
        print("    OS: \t" + device["operating_system"]["name"])
        for address in device["ip_addresses"]:
            if address["public"]:
                print("    IPv%s:\t%s" % (address["address_family"], address["address"]))
        print("    Created:\t" + device["created_at"].replace("T", " "))
        print("    Updated:\t" + device["updated_at"].replace("T", " "))
    assert response["meta"]["next"] is None


def api_request(path, json_data=None, method=None):
    request = urllib.request.Request("https://api.packet.net" + path, method=method)
    request.add_header("X-Auth-Token", PACKET_AUTH_TOKEN)
    if json_data is not None:
        request.add_header("Content-Type", "application/json")
        request.data = json.dumps(json_data)
    with urllib.request.urlopen(request) as response:
        return json.load(response)


if __name__ == "__main__":
    main(*sys.argv[1:])
