#!/usr/bin/env python3
"""Simple mock HTTP server for testing activity-http-generic.

This server echoes back request details for testing purposes.
"""

import json
import sys
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs

DEFAULT_PORT = 18083


class MockHTTPHandler(BaseHTTPRequestHandler):
    # Use HTTP/1.1 to support chunked transfer encoding
    protocol_version = 'HTTP/1.1'
    
    def read_body(self):
        """Read request body handling both Content-Length and chunked transfer."""
        content_length = self.headers.get('Content-Length')
        transfer_encoding = self.headers.get('Transfer-Encoding', '').lower()
        
        if content_length:
            return self.rfile.read(int(content_length)).decode('utf-8')
        elif 'chunked' in transfer_encoding:
            # Read chunked body
            chunks = []
            try:
                while True:
                    # Read chunk size line
                    size_line = self.rfile.readline()
                    if not size_line:
                        break
                    size_line = size_line.decode('utf-8').strip()
                    if not size_line:
                        continue
                    
                    # Parse chunk size (hex)
                    chunk_size = int(size_line.split(';')[0], 16)
                    if chunk_size == 0:
                        # End of chunks, read trailing headers/CRLF
                        self.rfile.readline()
                        break
                    
                    # Read chunk data
                    chunk_data = self.rfile.read(chunk_size)
                    chunks.append(chunk_data.decode('utf-8'))
                    
                    # Read trailing CRLF after chunk
                    self.rfile.readline()
                    
                return ''.join(chunks) if chunks else None
            except Exception as e:
                print(f"[MockHTTP] Error reading chunked body: {e}")
                return None
        return None

    def handle_request(self, method):
        try:
            parsed_path = urlparse(self.path)
            query = parse_qs(parsed_path.query)
            
            # Read body if present
            body = self.read_body()
            
            # Build response with request details
            response = {
                "method": method,
                "path": parsed_path.path,
                "query": {k: v[0] if len(v) == 1 else v for k, v in query.items()},
                "headers": {k.lower(): v for k, v in self.headers.items()},
                "body": body
            }
            
            # Special handling for specific paths
            if parsed_path.path == "/status/404":
                self.send_response(404)
                self.send_header('Content-Type', 'application/json')
                self.send_header('Connection', 'close')
                self.end_headers()
                self.wfile.write(json.dumps({"error": "Not found"}).encode())
                return
            
            if parsed_path.path == "/status/500":
                self.send_response(500)
                self.send_header('Content-Type', 'application/json')
                self.send_header('Connection', 'close')
                self.end_headers()
                self.wfile.write(json.dumps({"error": "Internal server error"}).encode())
                return
            
            if parsed_path.path == "/binary":
                data = bytes([0x00, 0x01, 0x02, 0x03, 0xFF])
                self.send_response(200)
                self.send_header('Content-Type', 'application/octet-stream')
                self.send_header('Content-Length', len(data))
                self.send_header('Connection', 'close')
                self.end_headers()
                self.wfile.write(data)
                return
            
            if parsed_path.path == "/empty":
                self.send_response(204)
                self.send_header('Connection', 'close')
                self.end_headers()
                return
            
            # Default: echo back JSON
            response_data = json.dumps(response).encode()
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.send_header('Content-Length', len(response_data))
            self.send_header('Connection', 'close')
            self.end_headers()
            self.wfile.write(response_data)
            
        except Exception as e:
            print(f"[MockHTTP] Error handling request: {e}")
            try:
                self.send_error(500, str(e))
            except:
                pass

    def do_GET(self):
        self.handle_request("GET")

    def do_POST(self):
        self.handle_request("POST")

    def do_PUT(self):
        self.handle_request("PUT")

    def do_DELETE(self):
        self.handle_request("DELETE")

    def do_PATCH(self):
        self.handle_request("PATCH")

    def do_HEAD(self):
        self.send_response(200)
        self.send_header('Content-Type', 'text/plain')
        self.send_header('X-Custom-Header', 'test-value')
        self.send_header('Connection', 'close')
        self.end_headers()

    def do_OPTIONS(self):
        self.send_response(200)
        self.send_header('Allow', 'GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS')
        self.send_header('Connection', 'close')
        self.end_headers()

    def log_message(self, format, *args):
        print(f"[MockHTTP] {args[0]}")


def main():
    port = int(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_PORT
    server = HTTPServer(('127.0.0.1', port), MockHTTPHandler)
    print(f"Mock HTTP server running on http://127.0.0.1:{port}")
    sys.stdout.flush()
    server.serve_forever()


if __name__ == '__main__':
    main()
