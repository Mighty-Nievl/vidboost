export PATH="$HOME/vidboost/bin:$HOME/.cargo/bin:$PATH"

export PORT=19000
export DATA_DIR="$HOME/vidboost/data"
export FFMPEG_PATH="$HOME/vidboost/bin/ffmpeg"
export FFPROBE_PATH="$HOME/vidboost/bin/ffprobe"
export AI_UPSCALER_BIN="$HOME/vidboost/scripts/fsrcnn-upscaler.sh"
export FSRCNN_MODEL_DIR="$HOME/vidboost/models"
export DRAWTEXT_FONTFILE="$HOME/vidboost/fonts/DejaVuSans.ttf"

# Pengaman CPU/RAM untuk VPS 2 vCPU
export FFMPEG_THREADS=2
export FSRCNN_THREADS=2
export MAX_OUTPUT_PIXELS=5000000
