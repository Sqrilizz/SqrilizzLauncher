#!/usr/bin/env bash
echo "Rust files:"
find src crates -name "*.rs" | xargs wc -l
