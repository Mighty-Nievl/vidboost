#!/usr/bin/env bash
# Wrapper supaya backend bisa memanggil Python seperti realesrgan-ncnn-vulkan.
exec python3 "$(dirname "$0")/fsrcnn_upscaler.py" "$@"
