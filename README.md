# dpf
**dpf** (dangoware picture format) is a simple image format designed 
for ease of encoding and decoding while maintaining a relatively good 
compression ratio for various purposes.

This reference implementation fits in around400 lines of relatively 
simple Rust, while maintaining decent decompression speeds.

Currently, only a lossless encoding scheme is supported. In the future,
a simple lossy encoding would be an interesting addition.
