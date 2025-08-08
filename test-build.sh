#!/bin/bash

# Default values
RESTORE=false

# Function to print usage
usage() {
    echo "Usage: $0 [options] [target]"
    echo "Options:"
    echo "  --restore                  Restore mode"
    exit 1
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --restore)
            RESTORE=true
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            if [[ -z "$TARGET" ]]; then
                TARGET="$1"
            fi
            shift
            ;;
    esac
done

docker compose down -v

if [[ "$RESTORE" != true ]]; then
    cp docker-compose.yml.build docker-compose.yml
    rm -rf .env

    docker login registry.gmedia.id
    docker build -t registry.gmedia.id/erp-api:rust -f ./Dockerfile .
    docker compose up -d
else
    cp docker-compose.yml.example docker-compose.yml
    cp .env.example .env
    docker compose up -d
fi
