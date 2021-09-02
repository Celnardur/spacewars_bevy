#!/usr/bin/env python3
import sys
from PIL import Image as im
from PIL import ImageFilter as flr

def blur(read_path, save_path, scale_factor, blur_radius):
    image = im.open(read_path)
    og_size = image.size
    resized = image.resize((og_size[0]*scale_factor, og_size[1]*scale_factor), resample=im.NEAREST)
    expanded = resized.filter(flr.MaxFilter(blur_radius))
    blured = expanded.filter(flr.BoxBlur(blur_radius))
    blured.save(save_path)

if __name__ == '__main__':
    scale_factor = 16
    blur_radius = 3
    blur_paths = [
        ("needle.png", "needle_blur.png"),
        ("needle_accelerating.png", "needle_acc_blur.png"),
        ("wedge.png", "wedge_blur.png"),
        ("wedge_accelerating.png", "wedge_acc_blur.png")
    ]

    for (load, save) in blur_paths:
        blur(load, save, scale_factor, blur_radius)


