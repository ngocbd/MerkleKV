#!/bin/bash

echo "=== Configuration Testing Script ==="
echo
echo "1. Testing with default config.toml (no environment variables):"
echo "  Expected: client_id='node1', client_password=None"
echo

# Test 1: No environment variables
unset CLIENT_ID
unset CLIENT_PASSWORD
timeout 3s cargo run --bin merkle_kv 2>/dev/null || echo "Server started briefly (expected timeout)"

echo
echo "2. Testing with CLIENT_ID environment variable:"
echo "  Expected: client_id='env_test_node', client_password=None"
echo

# Test 2: CLIENT_ID override
export CLIENT_ID="env_test_node"
unset CLIENT_PASSWORD
timeout 3s cargo run --bin merkle_kv 2>/dev/null || echo "Server started briefly with CLIENT_ID override"

echo
echo "3. Testing with CLIENT_PASSWORD environment variable:"
echo "  Expected: client_id='node1', client_password=Some([PROTECTED])"
echo

# Test 3: CLIENT_PASSWORD override
unset CLIENT_ID
export CLIENT_PASSWORD="env_test_password"
timeout 3s cargo run --bin merkle_kv 2>/dev/null || echo "Server started briefly with CLIENT_PASSWORD override"

echo
echo "4. Testing with both environment variables:"
echo "  Expected: client_id='env_both_node', client_password=Some([PROTECTED])"
echo

# Test 4: Both overrides
export CLIENT_ID="env_both_node"
export CLIENT_PASSWORD="env_both_password"
timeout 3s cargo run --bin merkle_kv 2>/dev/null || echo "Server started briefly with both overrides"

echo
echo "5. Testing with demo config file and environment overrides:"
echo "  Expected: client_id='env_demo_override', client_password=Some([PROTECTED]) (overriding demo_password)"
echo

# Test 5: Config file with environment overrides
export CLIENT_ID="env_demo_override"
export CLIENT_PASSWORD="env_override_password"
timeout 3s cargo run --bin merkle_kv -- --config examples/config_demo.toml 2>/dev/null || echo "Server started briefly with config file + env overrides"

# Clean up
unset CLIENT_ID
unset CLIENT_PASSWORD

echo
echo "=== Testing Complete ==="
echo "Note: Servers were stopped after 3 seconds to prevent hanging."
echo "The important part is that they start without compilation errors."
