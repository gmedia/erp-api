#!/bin/bash

cargo clean
rm -rf tarpaulin-report.html

cargo tarpaulin --force-clean --out Html --engine llvm --follow-exec \
    --exclude-files "api/src/openapi.rs" \
    --exclude-files "config/*" \
    --exclude-files "db/*" \
    --exclude-files "entity/*" \
    --exclude-files "migration/*" \
    --exclude-files "search/*" \
    --exclude-files "src/*"