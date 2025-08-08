#!/usr/bin/env python3
"""
Simple HTTP server with proper DDS MIME type support.
"""
import http.server
import socketserver
import mimetypes
import os

# Add DDS MIME type
mimetypes.add_type('application/octet-stream', '.dds')

PORT = 8080

class DdsHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        # Add CORS headers for WASM
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        super().end_headers()

if __name__ == "__main__":
    # Change to dist directory if it exists
    if os.path.exists('dist'):
        os.chdir('dist')

    with socketserver.TCPServer(("", PORT), DdsHTTPRequestHandler) as httpd:
        print(f"🚀 Serving at http://localhost:{PORT}")
        print("📁 DDS files will be served with correct MIME type")
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\n👋 Server stopped")
