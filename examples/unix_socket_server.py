#!/usr/bin/env python3
"""
Example Gunicorn server running over Unix domain socket

This simple Flask application demonstrates serving HTTP over Unix domain sockets
for use with Servo's Unix socket networking mode.

Usage:
    ./run_gunicorn.sh
"""

from flask import Flask, jsonify, request, render_template_string

app = Flask(__name__)

HTML_TEMPLATE = '''
<!DOCTYPE html>
<html>
<head>
    <title>Unix Socket Test - Servo Browser</title>
    <meta charset="utf-8">
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            max-width: 900px;
            margin: 50px auto;
            padding: 20px;
            background: #f5f5f5;
        }
        .container {
            background: white;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        h1 {
            color: #333;
            border-bottom: 3px solid #4CAF50;
            padding-bottom: 10px;
        }
        .info {
            background: #e8f4f8;
            padding: 20px;
            border-radius: 5px;
            margin: 20px 0;
            border-left: 4px solid #2196F3;
        }
        .success {
            background: #d4edda;
            border-left-color: #28a745;
            color: #155724;
        }
        .code {
            background: #f4f4f4;
            padding: 10px;
            border-radius: 3px;
            font-family: 'Courier New', monospace;
            font-size: 14px;
            overflow-x: auto;
        }
        a {
            color: #2196F3;
            text-decoration: none;
            padding: 8px 15px;
            background: #e3f2fd;
            border-radius: 4px;
            display: inline-block;
            margin: 5px;
        }
        a:hover {
            background: #2196F3;
            color: white;
        }
        ul {
            list-style: none;
            padding: 0;
        }
        li {
            margin: 10px 0;
        }
        .emoji {
            font-size: 24px;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1><span class="emoji">🎉</span> Hello from Unix Socket Server!</h1>

        <div class="info success">
            <h2>✅ Connection Successful</h2>
            <p><strong>This page was served over a Unix domain socket (IPC)!</strong></p>
            <p>No TCP connection was used - Servo accessed this web server through a Unix socket file.</p>
        </div>

        <div class="info">
            <h2>📡 Request Information</h2>
            <p><strong>Method:</strong> <span class="code">{{ method }}</span></p>
            <p><strong>Path:</strong> <span class="code">{{ path }}</span></p>
            <p><strong>User-Agent:</strong> <span class="code">{{ user_agent }}</span></p>
        </div>

        <div class="info">
            <h2>🔗 Test Links</h2>
            <ul>
                <li><a href="/api/data">📊 JSON API Endpoint</a></li>
                <li><a href="/test">🧪 Test Page</a></li>
                <li><a href="/about">ℹ️ About This Demo</a></li>
            </ul>
        </div>

        <div class="info">
            <h2>🚀 How It Works</h2>
            <ol>
                <li>Gunicorn serves this Flask app on a Unix domain socket</li>
                <li>Servo connects to the socket file instead of TCP</li>
                <li>HTTP requests/responses flow over IPC</li>
                <li>Zero network stack overhead for local development!</li>
            </ol>
        </div>
    </div>
</body>
</html>
'''

@app.route('/')
def index():
    return render_template_string(
        HTML_TEMPLATE,
        method=request.method,
        path=request.path,
        user_agent=request.headers.get('User-Agent', 'Unknown')
    )

@app.route('/api/data', methods=['GET', 'POST'])
def api_data():
    return jsonify({
        'status': 'success',
        'message': 'Hello from Unix Socket API',
        'transport': 'unix_domain_socket',
        'server': 'gunicorn + flask',
        'request': {
            'method': request.method,
            'path': request.path,
            'args': dict(request.args)
        },
        'headers': dict(request.headers)
    })

@app.route('/test')
def test_page():
    html = '''
    <!DOCTYPE html>
    <html>
    <head>
        <title>Test Page</title>
        <style>
            body { font-family: sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }
            .success { background: #d4edda; padding: 20px; border-radius: 5px; color: #155724; }
        </style>
    </head>
    <body>
        <h1>✓ Test Page</h1>
        <div class="success">
            <h2>Successfully loaded via Unix socket!</h2>
            <p>This demonstrates that Servo can load multiple pages from the same Unix socket server.</p>
        </div>
        <p><a href="/">← Back to Home</a></p>
    </body>
    </html>
    '''
    return html

@app.route('/about')
def about():
    html = '''
    <!DOCTYPE html>
    <html>
    <head>
        <title>About Unix Socket Demo</title>
        <style>
            body { font-family: sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }
            .info { background: #e8f4f8; padding: 20px; border-radius: 5px; margin: 20px 0; }
            code { background: #f4f4f4; padding: 2px 6px; border-radius: 3px; }
        </style>
    </head>
    <body>
        <h1>About This Demo</h1>
        <div class="info">
            <h2>Unix Domain Sockets (UDS)</h2>
            <p>Unix domain sockets provide inter-process communication (IPC) on the same machine
            without using the network stack. This is:</p>
            <ul>
                <li><strong>Faster</strong> - No TCP overhead</li>
                <li><strong>More secure</strong> - Filesystem permissions control access</li>
                <li><strong>Simpler</strong> - No port conflicts or firewall issues</li>
            </ul>
        </div>

        <div class="info">
            <h2>Servo Implementation</h2>
            <p>This modified version of Servo can connect to web servers over Unix sockets
            instead of TCP. The URL hostname maps to a socket file path.</p>
            <p>Example: <code>http://localhost/</code> connects to <code>/tmp/servo-sockets/localhost.sock</code></p>
        </div>

        <p><a href="/">← Back to Home</a></p>
    </body>
    </html>
    '''
    return html

if __name__ == '__main__':
    print("ERROR: Do not run this directly!")
    print("Use ./run_gunicorn.sh to start the server with Unix socket support")
    import sys
    sys.exit(1)
