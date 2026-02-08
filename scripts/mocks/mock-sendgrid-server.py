#!/usr/bin/env python3
"""Mock SendGrid API server for testing.

This server mocks the SendGrid Mail Send API:
- POST /v3/mail/send - Send an email
"""

import json
import sys
import uuid
from http.server import HTTPServer, BaseHTTPRequestHandler

DEFAULT_PORT = 18081


class MockSendGridHandler(BaseHTTPRequestHandler):
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
                print(f"[MockSendGrid] Error reading chunked body: {e}")
                return b'{}'
        return b'{}'
    
    def send_json_response(self, status_code, data=None, headers=None):
        response_data = json.dumps(data).encode() if data else b''
        self.send_response(status_code)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', len(response_data))
        self.send_header('Connection', 'close')
        if headers:
            for key, value in headers.items():
                self.send_header(key, value)
        self.end_headers()
        if response_data:
            self.wfile.write(response_data)

    def send_error_response(self, status_code, errors):
        error = {
            "errors": errors
        }
        self.send_json_response(status_code, error)

    def do_POST(self):
        if self.path == "/v3/mail/send":
            # Verify authorization header
            auth_header = self.headers.get('Authorization', '')
            if not auth_header.startswith('Bearer '):
                self.send_error_response(401, [{"message": "Authorization header missing or invalid"}])
                return

            body = self.read_body()

            try:
                request_data = json.loads(body) if body else {}
            except json.JSONDecodeError:
                self.send_error_response(400, [{"message": "Invalid JSON"}])
                return

            # Validate required fields
            errors = []
            if 'personalizations' not in request_data or not request_data['personalizations']:
                errors.append({"message": "The personalizations field is required.", "field": "personalizations"})
            if 'from' not in request_data:
                errors.append({"message": "The from field is required.", "field": "from"})
            if 'subject' not in request_data:
                errors.append({"message": "The subject field is required.", "field": "subject"})
            if 'content' not in request_data or not request_data['content']:
                errors.append({"message": "The content field is required.", "field": "content"})

            if errors:
                self.send_error_response(400, errors)
                return

            # Validate personalizations have recipients
            for i, p in enumerate(request_data['personalizations']):
                if 'to' not in p or not p['to']:
                    errors.append({"message": f"The to field is required in personalization {i}.", "field": f"personalizations.{i}.to"})

            if errors:
                self.send_error_response(400, errors)
                return

            # Generate a message ID and return 202 Accepted
            message_id = str(uuid.uuid4())
            self.send_json_response(202, None, {'x-message-id': message_id})
            return

        self.send_error_response(404, [{"message": f"Unknown endpoint: {self.path}"}])

    def log_message(self, format, *args):
        print(f"[MockSendGrid] {args[0]}")


def main():
    port = int(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_PORT
    server = HTTPServer(('127.0.0.1', port), MockSendGridHandler)
    print(f"Mock SendGrid API server running on http://127.0.0.1:{port}")
    sys.stdout.flush()
    server.serve_forever()


if __name__ == '__main__':
    main()
