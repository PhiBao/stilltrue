from http.server import BaseHTTPRequestHandler, HTTPServer

class H(BaseHTTPRequestHandler):
    def _deny(self):
        body = b'{"error":"deliberate drill outage"}'
        self.send_response(503)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)
    do_GET = do_POST = do_PUT = do_DELETE = _deny
    def log_message(self, fmt, *args):
        pass

HTTPServer(("0.0.0.0", 8000), H).serve_forever()
