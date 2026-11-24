#!/usr/bin/env python3
"""Multi-model upscaler kompatibel dengan argumen realesrgan-ncnn-vulkan.

Model diunduh ke AI_MODEL_DIR (default: ./models). Mendukung model
OpenCV dnn_superres (fsrcnn/espcn/edsr/lapsrn) dan binary eksternal
Real-ESRGAN (realesrgan-*) bila tersedia.
"""
import argparse
import os
import subprocess
import sys
import time

import cv2

MODEL_DIR = os.environ.get("AI_MODEL_DIR", os.environ.get("FSRCNN_MODEL_DIR", "./models"))

OPENCV_MODELS = {
    "fsrcnn": {2: "FSRCNN_x2.pb", 3: "FSRCNN_x3.pb", 4: "FSRCNN_x4.pb"},
    "espcn": {4: "ESPCN_x4.pb"},
    "edsr": {4: "EDSR_x4.pb"},
    "lapsrn": {8: "LapSRN_x8.pb"},
}

VULKAN_MODELS = {"realesr-animevideov3", "realesrgan-x4plus", "realesrgan-x4plus-anime"}


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser()
    p.add_argument("-i", "--input", required=True)
    p.add_argument("-o", "--output", required=True)
    p.add_argument("-n", "--model", default="fsrcnn")
    p.add_argument("-s", "--scale", type=int, default=2)
    p.add_argument("-t", "--tile", default="0")
    p.add_argument("-j", "--jobs", default="1:1:1")
    p.add_argument("-g", "--gpu", default="")
    p.add_argument("-m", "--model-path", default="")
    p.add_argument("-f", "--format", default="png")
    return p.parse_args()


def run_vulkan(args: argparse.Namespace) -> int:
    binary = os.environ.get("REALESRGAN_BIN", "")
    if not binary or not os.path.isfile(binary):
        print(
            "Model Vulkan butuh REALESRGAN_BIN. Unduh binary realesrgan-ncnn-vulkan "
            "dan set env tersebut (lihat docs/MODELS.md).",
            file=sys.stderr,
        )
        return 1
    cmd = [
        binary,
        "-i", args.input,
        "-o", args.output,
        "-n", args.model,
        "-s", str(args.scale),
        "-t", args.tile or "128",
        "-j", args.jobs or "1:1:1",
        "-f", "png",
        "-m", MODEL_DIR,
    ]
    if args.gpu:
        cmd += ["-g", args.gpu]
    result = subprocess.run(cmd, check=False)
    return result.returncode


def run_opencv(args: argparse.Namespace) -> int:
    algo = args.model
    variant = OPENCV_MODELS.get(algo)
    if not variant:
        print(f"Model OpenCV tidak dikenal: {algo}", file=sys.stderr)
        return 1

    scale = args.scale if args.scale in variant else sorted(variant)[-1]
    filename = variant[scale]
    model_path = os.path.join(MODEL_DIR, filename)
    if not os.path.isfile(model_path):
        print(f"Model belum terunduh: {model_path}", file=sys.stderr)
        return 1

    threads = int(os.environ.get("FSRCNN_THREADS", "4"))
    cv2.setNumThreads(max(1, threads))
    sr = cv2.dnn_superres.DnnSuperResImpl_create()
    sr.readModel(model_path)
    sr.setModel(algo, scale)

    os.makedirs(args.output, exist_ok=True)
    names = sorted(n for n in os.listdir(args.input) if n.lower().endswith(".png"))
    if not names:
        print("Tidak ada frame PNG di input", file=sys.stderr)
        return 1

    started = time.time()
    for index, name in enumerate(names, 1):
        src = os.path.join(args.input, name)
        dst = os.path.join(args.output, name)
        image = cv2.imread(src, cv2.IMREAD_COLOR)
        if image is None:
            continue
        cv2.imwrite(dst, sr.upsample(image))
        if index % 25 == 0 or index == len(names):
            print(f"{index}/{len(names)}", file=sys.stderr, flush=True)

    print(f"selesai dalam {time.time() - started:.1f}s", file=sys.stderr)
    return 0


def main() -> int:
    args = parse_args()
    if args.model in VULKAN_MODELS:
        return run_vulkan(args)
    return run_opencv(args)


if __name__ == "__main__":
    raise SystemExit(main())
