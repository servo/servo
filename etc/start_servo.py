"""
Created on Mon Mar 26 20:08:25 2018

@author: Pranshu Sinha, Abhay Soni, Aayushi Agrawal
The script is intended to start servo on localhost:7002
"""
import os
import subprocess

encoding = 'utf8'
p = None
def start_servo():
    global p
    cmds = ['cd ..', './mach run --webdriver 7002 --resolution 1024x768']
    p = subprocess.Popen('/bin/bash', stdin=subprocess.PIPE,
                 stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    
    for cmd in cmds:
        p.stdin.write(cmd + "\n")

def stop_servo():
    global p
    p.stdin.close()