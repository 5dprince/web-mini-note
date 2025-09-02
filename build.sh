#!/usr/bin/env bash
set -euo pipefail

IMAGE_NAME=${IMAGE_NAME:-web-mini-note-rust}
IMAGE_TAG=${IMAGE_TAG:-latest}

echo "Building docker image ${IMAGE_NAME}:${IMAGE_TAG}"
docker build -t "${IMAGE_NAME}:${IMAGE_TAG}" .

echo "Done. Run with: docker run -it --rm -p 8080:8080 -v $(pwd)/data:/app/_tmp ${IMAGE_NAME}:${IMAGE_TAG}"


