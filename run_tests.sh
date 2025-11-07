#!/bin/bash

set -e

echo "Running API tests..."
cargo test --test api_unit_tests --verbose

echo ""
echo "All tests passed successfully!"
