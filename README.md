# GPU Fucker

A Rust-based tool designed to stress test your system's GPU, CPU, and RAM simultaneously. This program uses OpenGL to render a computationally intensive Mandelbrot set on the GPU, spawns threads to max out CPU cores, and performs continuous memory operations to stress RAM.

**Warning**: This tool is intended to push your hardware to its limits. Use with caution, as it may cause overheating or system instability if run for extended periods. Ensure proper cooling and monitoring are in place.

---

## Features
- **GPU Stress**: Renders a Mandelbrot set with 1000 iterations per pixel using OpenGL shaders.
- **CPU Stress**: Spawns one thread per logical core to perform intensive floating-point calculations.
- **RAM Stress**: Allocates 512 MB of memory and continuously writes to it.
- **Simple Interface**: ASCII art instructions and keyboard controls (any key to start, 'q' to stop).

---

## Prerequisites
- **Rust**: Install the Rust toolchain from [rust-lang.org](https://www.rust-lang.org/).
- **OpenGL**: Ensure your system supports OpenGL 3.3 or higher.
- **Dependencies**: The project uses the following crates:
  - `glutin`: For window creation and event handling.
  - `gl`: For OpenGL bindings.
  - `colored`: For terminal text formatting.
  - `num_cpus`: To detect the number of CPU cores.
- A compatible OS (Windows, Linux, or macOS) with appropriate graphics drivers.

---

## Installation
1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/gpu-stresser.git
   cd gpu-stresser
