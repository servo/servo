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
            max-width: 1000px;
            margin: 20px auto;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        }
        .container {
            background: white;
            padding: 40px;
            border-radius: 15px;
            box-shadow: 0 10px 40px rgba(0,0,0,0.3);
        }
        .banner {
            background: linear-gradient(135deg, #11998e 0%, #38ef7d 100%);
            color: white;
            padding: 30px;
            border-radius: 10px;
            text-align: center;
            margin-bottom: 30px;
            font-size: 28px;
            font-weight: bold;
            box-shadow: 0 5px 20px rgba(0,0,0,0.2);
        }
        .banner .big-emoji {
            font-size: 64px;
            display: block;
            margin-bottom: 10px;
        }
        h1 {
            color: #333;
            border-bottom: 3px solid #4CAF50;
            padding-bottom: 10px;
        }
        h2 {
            color: #444;
            margin-top: 0;
        }
        .info {
            background: #e8f4f8;
            padding: 20px;
            border-radius: 8px;
            margin: 20px 0;
            border-left: 5px solid #2196F3;
        }
        .success {
            background: #d4edda;
            border-left-color: #28a745;
            color: #155724;
        }
        .warning {
            background: #fff3cd;
            border-left-color: #ffc107;
            color: #856404;
        }
        .code {
            background: #2d2d2d;
            color: #f8f8f2;
            padding: 12px;
            border-radius: 5px;
            font-family: 'Courier New', monospace;
            font-size: 13px;
            overflow-x: auto;
            margin: 10px 0;
        }
        .inline-code {
            background: #f4f4f4;
            padding: 2px 6px;
            border-radius: 3px;
            font-family: 'Courier New', monospace;
            font-size: 14px;
        }
        table {
            width: 100%;
            border-collapse: collapse;
            margin: 15px 0;
        }
        th, td {
            padding: 10px;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }
        th {
            background: #f8f9fa;
            font-weight: bold;
            color: #333;
        }
        tr:hover {
            background: #f8f9fa;
        }
        a {
            color: #2196F3;
            text-decoration: none;
            padding: 10px 20px;
            background: #e3f2fd;
            border-radius: 5px;
            display: inline-block;
            margin: 5px;
            font-weight: 500;
            transition: all 0.3s;
        }
        a:hover {
            background: #2196F3;
            color: white;
            transform: translateY(-2px);
            box-shadow: 0 5px 15px rgba(33, 150, 243, 0.3);
        }
        ul {
            list-style: none;
            padding: 0;
        }
        li {
            margin: 10px 0;
        }
        .emoji {
            font-size: 20px;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="banner">
            <span class="big-emoji">🚀✨</span>
            UNIX DOMAIN SOCKET CONNECTION SUCCESSFUL!
            <div style="font-size: 18px; margin-top: 10px; font-weight: normal;">
                You are viewing this page through IPC, not TCP/IP!
            </div>
        </div>

        <div class="info success">
            <h2>✅ What Just Happened?</h2>
            <p><strong>Servo connected to this web server using a Unix domain socket!</strong></p>
            <p>This means:</p>
            <ul style="margin-left: 20px;">
                <li>✓ Zero TCP/IP networking</li>
                <li>✓ Direct inter-process communication</li>
                <li>✓ Socket file: <span class="inline-code">/tmp/servo-sockets/localhost.sock</span></li>
                <li>✓ Faster and more secure than TCP</li>
            </ul>
        </div>

        <div class="info">
            <h2>📡 HTTP Request Details</h2>
            <table>
                <tr>
                    <th>Property</th>
                    <th>Value</th>
                </tr>
                <tr>
                    <td><strong>Method</strong></td>
                    <td><span class="inline-code">{{ method }}</span></td>
                </tr>
                <tr>
                    <td><strong>Path</strong></td>
                    <td><span class="inline-code">{{ path }}</span></td>
                </tr>
                <tr>
                    <td><strong>Query String</strong></td>
                    <td><span class="inline-code">{{ query_string }}</span></td>
                </tr>
                <tr>
                    <td><strong>Remote Addr</strong></td>
                    <td><span class="inline-code">{{ remote_addr }}</span></td>
                </tr>
            </table>
        </div>

        <div class="info">
            <h2>📋 All Request Headers</h2>
            <table>
                <tr>
                    <th>Header Name</th>
                    <th>Value</th>
                </tr>
                {% for header_name, header_value in headers %}
                <tr>
                    <td><strong>{{ header_name }}</strong></td>
                    <td><span class="inline-code">{{ header_value }}</span></td>
                </tr>
                {% endfor %}
            </table>
        </div>

        <div class="info">
            <h2>🔗 Test Links</h2>
            <ul>
                <li><a href="/">🏠 Home Page</a></li>
                <li><a href="/api/data">📊 JSON API Endpoint</a></li>
                <li><a href="/test">🧪 Test Page</a></li>
                <li><a href="/about">ℹ️ About This Demo</a></li>
            </ul>
        </div>

        <div class="info warning">
            <h2>⚙️ How It Works</h2>
            <ol style="margin-left: 20px;">
                <li>Gunicorn binds to <span class="inline-code">/tmp/servo-sockets/localhost.sock</span></li>
                <li>Servo maps <span class="inline-code">localhost</span> → socket file</li>
                <li>When you visit <span class="inline-code">http://localhost/</span>, Servo connects to the socket</li>
                <li>HTTP flows over Unix IPC instead of TCP/IP!</li>
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
        query_string=request.query_string.decode('utf-8') if request.query_string else '(none)',
        remote_addr=request.remote_addr or '(Unix socket - no remote addr)',
        headers=request.headers.items()
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
