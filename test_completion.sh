#!/bin/bash
# Test directory completion manually

# Create test directories
mkdir -p /tmp/test_completion/rat/dog
cd /tmp/test_completion

# Run the shell
echo "Created test directory structure:"
ls -laR
echo ""
echo "Running shell..."
echo "Type 'du ' and press TAB twice to test"
/Users/mieky/Job/self/codecrafters-shell-rust/your_program.sh
