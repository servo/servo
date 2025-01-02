from os import path, listdir
from hashlib import sha512, sha256, md5
from base64 import b64encode
import re

JS_DIR = path.normpath(path.join(__file__, "..", ".."))

'''
Yield each file in the javascript directory
'''
def js_files():
  for f in listdir(JS_DIR):
    if path.isfile(f) and f.endswith(".js"):
      yield f

'''
URL-safe base64 encode a binary digest and strip any padding.
'''
def format_digest(digest):
  return b64encode(digest)

'''
Generate an encoded sha512 URI.
'''
def sha512_uri(content):
  return "sha512-%s" % format_digest(sha512(content).digest())

'''
Generate an encoded sha256 URI.
'''
def sha256_uri(content):
  return "sha256-%s" % format_digest(sha256(content).digest())

'''
Generate an encoded md5 digest URI.
'''
def md5_uri(content):
  return "md5-%s" % format_digest(md5(content).digest())

def main():
  for file in js_files():
    print("Generating content for %s" % file)
    base = path.splitext(path.basename(file))[0]
    var_name = re.sub(r"[^a-z0-9]", "_", base)
    content = "%s=true;" % var_name
    with open(file, "w") as f: f.write(content)
    print("\tSHA512 integrity: %s" % sha512_uri(content))
    print("\tSHA256 integrity: %s" % sha256_uri(content))
    print("\tMD5 integrity:    %s" % md5_uri(content))

if __name__ == "__main__":
  main()
