<p align="center">
  <img width="400px" src="https://github.com/user-attachments/assets/98f94c1c-ed6f-49a3-b906-c328035d981e">
</p>

# SQP
**SQP** (Squishy Picture) is a simple image format designed 
for ease of encoding and decoding while maintaining a relatively good 
compression ratio for various purposes. The general idea is to make
something "good enough" while being simple.

This reference implementation fits in under 1000 lines of relatively 
simple Rust, while maintaining decent compression and decompression
speeds.

Additionally, it also supports both lossless and lossy encoding schemes,
with the lossy version using Discrete Cosine Transform encoding like JPEG.
