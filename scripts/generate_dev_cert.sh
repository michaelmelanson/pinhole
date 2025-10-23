#!/bin/bash

# Generate self-signed certificate for local development
# This certificate will be valid for 365 days for localhost and 127.0.0.1

set -e

echo "Generating self-signed TLS certificate for development..."

# Generate private key and certificate in one command
openssl req -x509 \
    -newkey rsa:4096 \
    -keyout key.pem \
    -out cert.pem \
    -days 365 \
    -nodes \
    -subj "/C=US/ST=Development/L=Local/O=Pinhole/CN=localhost" \
    -addext "subjectAltName=DNS:localhost,IP:127.0.0.1"

echo ""
echo "✓ Certificate generation complete!"
echo ""
echo "Generated files:"
echo "  - cert.pem (certificate)"
echo "  - key.pem (private key)"
echo ""
echo "These files are valid for 365 days and work with localhost/127.0.0.1"
echo ""
echo "IMPORTANT: These are self-signed certificates for development only."
echo "           Do NOT use in production!"
echo ""
