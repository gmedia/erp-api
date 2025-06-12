#!/bin/bash

cargo tarpaulin --all-targets --ignore-tests --out Html \
    --exclude-files "migrations/*" \
    --exclude-files "src/config/*" \
    --exclude-files "src/api/openapi.rs" \
    --exclude-files "src/main.rs"