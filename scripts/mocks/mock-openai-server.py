#!/usr/bin/env python3
"""Mock OpenAI Responses API server for testing.

This server mocks the OpenAI Responses API endpoints:
- POST /v1/responses - Create a response
- GET /v1/responses/:id - Get a response
- DELETE /v1/responses/:id - Delete a response
- GET /v1/responses - List responses
- GET /v1/responses/:id/input_items - List input items
- POST /v1/responses/:id/cancel - Cancel a response
"""

import json
import sys
import time
import uuid
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs

DEFAULT_PORT = 18080

# In-memory storage for responses
responses_store = {}


def generate_response_id():
    return f"resp_{uuid.uuid4().hex[:24]}"


def create_mock_response(request_data):
    """Create a mock response based on the request."""
    response_id = generate_response_id()
    model = request_data.get('model', 'gpt-4o-mini')
    input_data = request_data.get('input', [])
    
    # Handle simple string input
    if isinstance(input_data, str):
        input_text = input_data
    elif isinstance(input_data, list) and len(input_data) > 0:
        # Extract text from input messages
        input_text = ""
        for msg in input_data:
            if isinstance(msg, dict):
                content = msg.get('content', [])
                if isinstance(content, list):
                    for c in content:
                        if isinstance(c, dict) and c.get('type') == 'input_text':
                            input_text = c.get('text', '')
                            break
                        elif isinstance(c, str):
                            input_text = c
                            break
    else:
        input_text = "test input"
    
    now = int(time.time())
    
    response = {
        "id": response_id,
        "object": "response",
        "created_at": now,
        "status": "completed",
        "model": model,
        "output": [
            {
                "type": "message",
                "id": f"msg_{uuid.uuid4().hex[:24]}",
                "status": "completed",
                "role": "assistant",
                "content": [
                    {
                        "type": "output_text",
                        "text": f"Mock response for: {input_text[:100]}",
                        "annotations": []
                    }
                ]
            }
        ],
        "parallel_tool_calls": True,
        "previous_response_id": request_data.get('previous_response_id'),
        "reasoning": request_data.get('reasoning'),
        "store": request_data.get('store', True),
        "temperature": request_data.get('temperature', 1.0),
        "text": {
            "format": {
                "type": "text"
            }
        },
        "tool_choice": "auto",
        "tools": [],
        "top_p": request_data.get('top_p', 1.0),
        "truncation": "disabled",
        "usage": {
            "input_tokens": 50,
            "output_tokens": 20,
            "total_tokens": 70,
            "output_tokens_details": {
                "reasoning_tokens": 0
            }
        },
        "user": request_data.get('user'),
        "metadata": request_data.get('metadata', {}),
        "output_text": f"Mock response for: {input_text[:100]}"
    }
    
    # Store the response
    responses_store[response_id] = response
    
    return response


class MockOpenAIHandler(BaseHTTPRequestHandler):
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
                print(f"[MockOpenAI] Error reading chunked body: {e}")
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

    def send_error_response(self, status_code, message, error_type="invalid_request_error"):
        error = {
            "error": {
                "message": message,
                "type": error_type,
                "code": None
            }
        }
        self.send_json_response(status_code, error)

    def do_POST(self):
        parsed_path = urlparse(self.path)
        path = parsed_path.path
        
        body = self.read_body()
        
        try:
            request_data = json.loads(body) if body else {}
        except json.JSONDecodeError:
            self.send_error_response(400, "Invalid JSON in request body")
            return

        # POST /v1/responses - Create a response
        if path == "/v1/responses" or path == "/responses":
            if 'model' not in request_data:
                self.send_error_response(400, "Missing required parameter: model")
                return
            if 'input' not in request_data:
                self.send_error_response(400, "Missing required parameter: input")
                return
            
            response = create_mock_response(request_data)
            self.send_json_response(200, response)
            return

        # POST /v1/responses/:id/cancel - Cancel a response
        cancel_pattern = path.endswith("/cancel") and ("/v1/responses/" in path or "/responses/" in path)
        if cancel_pattern:
            parts = path.split("/")
            response_id = parts[-2]
            if response_id in responses_store:
                response = responses_store[response_id].copy()
                response['status'] = 'cancelled'
                responses_store[response_id] = response
                self.send_json_response(200, response)
            else:
                self.send_error_response(404, f"Response {response_id} not found")
            return

        self.send_error_response(404, f"Unknown endpoint: {path}")

    def do_GET(self):
        parsed_path = urlparse(self.path)
        path = parsed_path.path
        query = parse_qs(parsed_path.query)

        # GET /v1/responses - List responses
        if path == "/v1/responses" or path == "/responses":
            limit = int(query.get('limit', [20])[0])
            order = query.get('order', ['desc'])[0]
            after = query.get('after', [None])[0]
            before = query.get('before', [None])[0]
            
            responses = list(responses_store.values())
            if order == 'asc':
                responses.sort(key=lambda x: x['created_at'])
            else:
                responses.sort(key=lambda x: x['created_at'], reverse=True)
            
            responses = responses[:limit]
            
            result = {
                "object": "list",
                "data": responses,
                "first_id": responses[0]['id'] if responses else None,
                "last_id": responses[-1]['id'] if responses else None,
                "has_more": len(responses_store) > limit
            }
            self.send_json_response(200, result)
            return

        # GET /v1/responses/:id/input_items - List input items
        if "/input_items" in path:
            parts = path.split("/")
            response_id = parts[-2]
            if response_id in responses_store:
                result = {
                    "object": "list",
                    "data": [],
                    "first_id": None,
                    "last_id": None,
                    "has_more": False
                }
                self.send_json_response(200, result)
            else:
                self.send_error_response(404, f"Response {response_id} not found")
            return

        # GET /v1/responses/:id - Get a response
        if path.startswith("/v1/responses/") or path.startswith("/responses/"):
            parts = path.split("/")
            response_id = parts[-1]
            if response_id in responses_store:
                self.send_json_response(200, responses_store[response_id])
            else:
                self.send_error_response(404, f"Response {response_id} not found")
            return

        self.send_error_response(404, f"Unknown endpoint: {path}")

    def do_DELETE(self):
        parsed_path = urlparse(self.path)
        path = parsed_path.path

        # DELETE /v1/responses/:id - Delete a response
        if path.startswith("/v1/responses/") or path.startswith("/responses/"):
            parts = path.split("/")
            response_id = parts[-1]
            if response_id in responses_store:
                del responses_store[response_id]
                self.send_json_response(200, {"id": response_id, "object": "response.deleted", "deleted": True})
            else:
                self.send_error_response(404, f"Response {response_id} not found")
            return

        self.send_error_response(404, f"Unknown endpoint: {path}")

    def log_message(self, format, *args):
        print(f"[MockOpenAI] {args[0]}")


def main():
    port = int(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_PORT
    server = HTTPServer(('127.0.0.1', port), MockOpenAIHandler)
    print(f"Mock OpenAI Responses API server running on http://127.0.0.1:{port}")
    print(f"Use OPENAI_API_BASE_URL=http://127.0.0.1:{port} for testing")
    sys.stdout.flush()
    server.serve_forever()


if __name__ == '__main__':
    main()
