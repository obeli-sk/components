#!/usr/bin/env python3
"""Mock Postmark API server for testing.

This server mocks the Postmark Email API:
- POST /email - Send an email
"""

import json
import sys
import uuid
from datetime import datetime, timezone
from http.server import HTTPServer, BaseHTTPRequestHandler

DEFAULT_PORT = 18082


class MockPostmarkHandler(BaseHTTPRequestHandler):
    protocol_version = 'HTTP/1.1'
    
    def read_body(self):
        """Read request body handling both Content-Length and chunked transfer."""
        content_length = self.headers.get('Content-Length')
        transfer_encoding = self.headers.get('Transfer-Encoding', '').lower()
        
        if content_length:
            return self.rfile.read(int(content_length))
        elif 'chunked' in transfer_encoding:
            chunks = []
            try:
                while True:
                    size_line = self.rfile.readline()
                    if not size_line:
                        break
                    size_line = size_line.decode('utf-8').strip()
                    if not size_line:
                        continue
                    chunk_size = int(size_line.split(';')[0], 16)
                    if chunk_size == 0:
                        self.rfile.readline()
                        break
                    chunk_data = self.rfile.read(chunk_size)
                    chunks.append(chunk_data)
                    self.rfile.readline()
                return b''.join(chunks) if chunks else b'{}'
            except Exception as e:
                print(f"[MockPostmark] Error reading chunked body: {e}")
                return b'{}'
        return b'{}'
    
    def send_json_response(self, status_code, data):
        response_data = json.dumps(data).encode()
        self.send_response(status_code)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', len(response_data))
        self.send_header('Connection', 'close')
        self.end_headers()
        self.wfile.write(response_data)

    def do_POST(self):
        if self.path == "/email":
            # Verify server token header
            server_token = self.headers.get('X-Postmark-Server-Token', '')
            if not server_token:
                self.send_json_response(401, {
                    "ErrorCode": 10,
                    "Message": "Please provide a valid Server Token"
                })
                return

            body = self.read_body()

            try:
                request_data = json.loads(body) if body else {}
            except json.JSONDecodeError:
                self.send_json_response(400, {
                    "ErrorCode": 0,
                    "Message": "Invalid JSON"
                })
                return

            # Validate required fields
            if 'From' not in request_data:
                self.send_json_response(422, {
                    "ErrorCode": 300,
                    "Message": "You must specify a from address."
                })
                return

            if 'To' not in request_data or not request_data['To']:
                self.send_json_response(422, {
                    "ErrorCode": 300,
                    "Message": "You must specify a recipient."
                })
                return

            if 'Subject' not in request_data:
                self.send_json_response(422, {
                    "ErrorCode": 300,
                    "Message": "You must specify a subject."
                })
                return

            if 'TextBody' not in request_data and 'HtmlBody' not in request_data:
                self.send_json_response(422, {
                    "ErrorCode": 300,
                    "Message": "You must provide either text or HTML body (or both)."
                })
                return

            # Success response
            message_id = str(uuid.uuid4())
            submitted_at = datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%S.%f')[:-3] + 'Z'
            
            self.send_json_response(200, {
                "To": request_data['To'],
                "SubmittedAt": submitted_at,
                "MessageID": message_id,
                "ErrorCode": 0,
                "Message": "OK"
            })
            return

        self.send_json_response(404, {
            "ErrorCode": 0,
            "Message": f"Unknown endpoint: {self.path}"
        })

    def log_message(self, format, *args):
        print(f"[MockPostmark] {args[0]}")


def main():
    port = int(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_PORT
    server = HTTPServer(('127.0.0.1', port), MockPostmarkHandler)
    print(f"Mock Postmark API server running on http://127.0.0.1:{port}")
    sys.stdout.flush()
    server.serve_forever()


if __name__ == '__main__':
    main()
