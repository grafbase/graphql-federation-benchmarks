#!/bin/bash
# Quick test to verify report generation

# Clean up any previous test files
rm -f REPORT.md
rm -rf charts/

# Run a quick benchmark (it will fail because we don't have all the infrastructure, but that's okay)
echo "Testing report generation..."
cargo run -p cli -- run 2>/dev/null || true

# Check if REPORT.md was created
if [ -f "REPORT.md" ]; then
    echo "✓ REPORT.md was created"
    echo "First 10 lines of REPORT.md:"
    head -10 REPORT.md
else
    echo "✗ REPORT.md was NOT created"
fi

# Check if charts directory was created
if [ -d "charts" ]; then
    echo "✓ charts/ directory was created"
    ls charts/ 2>/dev/null && echo "Chart files found" || echo "No chart files (expected if no benchmarks completed)"
else
    echo "✗ charts/ directory was NOT created"
fi

# Clean up
rm -f REPORT.md
rm -rf charts/