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
The script is intended to start servo on localhost:7002
"""
import subprocess


def start_servo(port, resolution):

    # Use the below command if you are running this script on windows
    # cmds = 'mach.bat run --webdriver ' + port + ' --resolution ' + resolution
    cmds = './mach run --webdriver=' + port + ' --resolution ' + resolution
    process = subprocess.Popen(cmds, shell=True, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    return process
