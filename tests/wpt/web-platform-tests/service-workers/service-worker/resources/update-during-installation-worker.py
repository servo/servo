import random

def main(request, response):
    headers = [(b'Content-Type', b'application/javascript'),
               (b'Cache-Control', b'max-age=0')]
    # Plug in random.random() to the worker so update() finds a new worker every time.
    body = u'''
// %s
importScripts('update-during-installation-worker.js');
    '''.strip() % (random.random())
    return headers, body
