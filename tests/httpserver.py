import SimpleHTTPServer
import SocketServer

PORT = 8000

Handler = SimpleHTTPServer.SimpleHTTPRequestHandler

class MyTCPServer(SocketServer.TCPServer):
    request_queue_size = 2000
    allow_reuse_address = True

httpd = MyTCPServer(("", PORT), Handler)

print "serving at port", PORT
httpd.serve_forever()
