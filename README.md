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
- Lossy alpha compression!
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

## Examples
All examples are at 30% quality in both JPEG and SQP.

| Original | JPEG | SQP |
|----------|--------------|-------------|
| <img width="300px" src="https://github.com/user-attachments/assets/e4f7b620-4cf5-407d-851b-800c52c8a14d"> | <img width="300px" src="https://github.com/user-attachments/assets/84691e8c-2f73-4a1d-b979-0863066b159f"> | <img width="300px" src="https://github.com/user-attachments/assets/ccaa8770-b641-437f-80d1-3658f94c2e21"> |
| <img width="300px" src="https://github.com/user-attachments/assets/f0056e3b-8988-4d0d-88bf-bc73ac5b8be0"> | <img width="300px" src="https://github.com/user-attachments/assets/400c4072-ba69-45d7-8051-46a4e2867c7f"> | <img width="300px" src="https://github.com/user-attachments/assets/c4c84f64-7564-433a-a922-17da472578d9"> |

Images obtained from the following source:
[https://r0k.us/graphics/kodak/](https://r0k.us/graphics/kodak/)
