#!/bin/bash

rm -rf tarpaulin-report.html

cargo tarpaulin --force-clean --workspace --follow-exec --engine llvm --out Html \
    --exclude-files "api/src/openapi.rs" \
    --exclude-files "config/*" \
    --exclude-files "db/*" \
    --exclude-files "entity/*" \
    --exclude-files "migration/*" \
    --exclude-files "search/*" \
    --exclude-files "src/*"