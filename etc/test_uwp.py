#!/usr/bin/env python

# Copyright 2019 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

# Usage: python etc/test_uwp.py
#
# Install and run the UWP Servo app. To make sure the app is functional,
# a httpd server is started and runs until a request is made by Servo.

import os
from subprocess import check_call, CalledProcessError
from BaseHTTPServer import BaseHTTPRequestHandler, HTTPServer
from threading import Thread

# Files
app_name = 'MozillaFoundation.FirefoxReality'
path = 'support\\hololens\\AppPackages\\ServoApp\\ServoApp_1.0.0.0_Debug_Test\\'
appx_file = os.getcwd() + '\\' + path + 'ServoApp_1.0.0.0_x64_Debug.msixbundle'
dep_file = os.getcwd() + '\\' + path + 'Dependencies\\x64\\Microsoft.VCLibs.x64.Debug.14.00.appx'

if not os.path.isfile(appx_file):
    print "Can't find ServoApp package (was `mach package` run?)"
    exit(1)

def run_powershell_cmd_dont_fail(cmd):
    try:
        print "Running PowerShell command: ", cmd
        check_call(['powershell.exe', '-NoProfile', '-Command', cmd])
    except CalledProcessError:
        print "ERROR: PowerShell command failed"
        check_call(['powershell.exe', '-NoProfile', '-Command', 'Get-Appxlog'])

def run_powershell_cmd(cmd):
    try:
        print "Running PowerShell command: ", cmd
        check_call(['powershell.exe', '-NoProfile', '-Command', cmd])
    except CalledProcessError:
        print "ERROR: PowerShell command failed"
        check_call(['powershell.exe', '-NoProfile', '-Command', 'Get-Appxlog'])
        exit(1)


def start_httpd(port):
    class handler(BaseHTTPRequestHandler):
        def do_GET(self):
            self.send_response(200)
            self.send_header('Content-type', 'text/html')
            self.end_headers()
            self.wfile.write("Hello")
            print "Got HTTP request. Shutting down HTTP Server."
            Thread(target=server.shutdown).start()
            return
    server = HTTPServer(('', port), handler)
    thread = Thread(target=server.serve_forever)
    print "Starting HTTP server."
    thread.start()
    return server, thread

app_family = '$(Get-AppxPackage ' + app_name + '| select -expandproperty PackageFamilyName)'
uninstall_cmd = '$(Get-AppxPackage ' + app_name + ')| Remove-AppxPackage'

# Installing app
# Uninstalling first. Just in case.
run_powershell_cmd(uninstall_cmd)

run_powershell_cmd_dont_fail('dir cert: -Recurse | Where-Object {$_.Issuer -eq "CN=Allizom"}')
run_powershell_cmd_dont_fail('Get-AuthenticodeSignature -FilePath ' + appx_file + ' | Select-Object *')
c1 = 'reg query "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\AppModelUnlock" /v "AllowDevelopmentWithoutDevLicense"'
c2 = 'reg query "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\AppModelUnlock" /v "AllowAllTrustedApps"'
run_powershell_cmd_dont_fail(c1)
run_powershell_cmd_dont_fail(c2)
run_powershell_cmd('Add-AppxPackage -Path ' + dep_file)
run_powershell_cmd('Add-AppxPackage -Path ' + appx_file)
# Allow app to connect to localhost
checknetisolation = 'checknetisolation loopbackexempt {} -n="' + app_family + '"'
run_powershell_cmd(checknetisolation.format('-a'))

# HTTP Server
port = 56012
# Starting HTTPD server
http_server, http_thread = start_httpd(port)

# Running Servo via its protocol handler
url = "http://localhost:" + str(port)

run_powershell_cmd('Start-Process -ArgumentList ' + url + ' shell:AppsFolder\\' + app_family + '!App')

http_thread.join(timeout=120)
success = True
if http_thread.is_alive():
    http_server.shutdown()
    print "Error: Timeout"
    success = False

# Resetting localhost access permissions
run_powershell_cmd(checknetisolation.format('-d'))

# Stopping and uninstalling app
run_powershell_cmd(uninstall_cmd)

exit(0 if success else 1)
