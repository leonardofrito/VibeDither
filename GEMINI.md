# GEMINI.md - VibeDither Project Context

## Project Overview
**VibeDither** is a modern, futuristic, and minimalistic image dithering application designed for Windows 11. It features a terminal-like aesthetic and provides high-performance, GPU-accelerated image processing tools for artists and designers.

### Main Technologies
- **Language:** Rust
- **GPU Graphics/Compute:** `wgpu` (DirectX 12 backend)
- **UI Framework:** `egui` (Matrix Green Terminal style)
- **Image Processing:** 16-bit floating-point internal pipeline

## Architecture & Features
1. **Core Pipeline:**
    - Non-destructive image processing.
    - Strict 16-byte uniform alignment for high-performance GPU updates.
    - Fixed order: Basic Adjustments -> Posterization -> Dithering -> RGB Curves.
2. **Key UI Elements:**
    - Matrix Green terminal-inspired layout with monospace fonts.
    - Left control panel with toggleable tabs (Adjust, Dither).
    - Custom Spline-based RGB Curves editor.
    - Real-time live preview with Zoom and Fit-to-Screen support.
3. **Current Features:**
    - **Adjustments:** Exposure, Contrast, Highlights, Shadows, Whites, Blacks, Temperature, Tint, Vibrance, Saturation, Sharpness, and Brightness.
    - **Posterization:** Quantization applied before dithering for better transitions.
    - **Dithering:** 
        - Threshold (with adjustable threshold).
        - Random (White Noise).
        - Bayer (Ordered) with selectable Matrix Sizes: 2x2, 3x3, 4x4, 8x8.
        - Advanced modes: Blue Noise, Stucki, Atkinson, Gradient Based, Lattice-Boltzmann.
        - Adjustable Pixel Scale (Resolution Scaling).
    - **I/O:** Drag & drop, Clipboard (Paste), and System File Picker.

## Status: 2026-02-12
- [x] Disable linear pixel filtering (Nearest Neighbor for dither clarity).
- [x] Advanced Color Dithering (Blue Noise, Stucki, Atkinson, Gradient, Lattice-Boltzmann).
- [x] Gradient Remap multi-stop system.
- [x] Integrated Posterize as a multi-level dither effect.
- [ ] Image Export (Save to file/Copy to clipboard).

## Recent Achievements
- Switched GPU sampler to `Nearest` for sharp pixel art/dither aesthetic.
- Implemented a wide range of dithering approximations optimized for GPU fragment shaders.
- Created a robust 1D texture-based Gradient Remap system with a custom UI for managing color stops.
- Moved Posterization to the Dither pipeline, allowing for sophisticated multi-level dither patterns.
