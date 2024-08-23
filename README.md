<p align="center">
  <img title="SQP" alt="SQP Logo" width="500px" src="https://github.com/user-attachments/assets/85abfb2f-6240-42d7-bdef-2103d6f83765">
</p>

**SQP** (**SQ**uishy **P**icture Format) is an image format designed 
for ease of implementation and learning about compression and image formats
while attaining a relatively good compression ratio. The general idea is to
make something "good enough" while being simple, and also as a learning tool
to learn about compression (mostly on my part). If you need an image format
for general use, this is probably **not it**, go check out JPEG XL or AVIF.

This reference implementation fits in around 1000 lines of relatively 
simple Rust, while maintaining decent compression and decompression
speeds.

## Features
- Lossless and lossy compression schemes
- Support for various color formats (RGBA, Grayscale, etc.)
- Decent compression ratios, the lossless compression can often beat PNG
  especially on images with transparency
- Relatively simple
- Squishy! 🍡

## Future Features
- Animated images
  - Frame difference encoding
  - Loop points
  - Arbitrary frame timings
  - Decoder-based frame interpolation
- Floating point color
- Metadata?
