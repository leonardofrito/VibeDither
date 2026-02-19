# GEMINI.md - VibeDither Project Context

## Project Overview
**VibeDither** is a modern, futuristic, and minimalistic image dithering application designed for Windows 11. It features a high-contrast terminal aesthetic and provides high-performance, GPU-accelerated image processing tools for artists and designers. 

**Note:** The application is strictly **image-only**. All video functionality has been removed to focus on pure static image reconstruction and dithering.

### Main Technologies
- **Language:** Rust
- **GPU Graphics/Compute:** `wgpu` (DirectX 12 backend)
- **UI Framework:** `egui` (Matrix Green Terminal style: RGB 0, 255, 0 on RGB 0, 0, 0)
- **Typography:** Cascadia Mono
- **Image Processing:** 16-bit floating-point internal pipeline

## Architecture & Features
1. **Core Pipeline:**
    - Non-destructive image processing.
    - Strict 16-byte uniform alignment for high-performance GPU updates.
    - Fixed order: Basic Adjustments -> Dithering (with internal Posterization) -> Gradient Remap.
2. **Key UI Elements:**
    - Matrix Green terminal-inspired layout with monochromatic palette.
    - Tactical Keyboard-centric navigation (Hierarchical Menus with [MODE] labels).
    - Centered "Oscilloscope" style editing overlay for parameter changes.
    - High-fidelity RGB Curves editor with 4x4 background grid.
    - Blender-style Color Ramp for Gradient Remap with hex/RGB copy support.
    - Real-time live preview with Zoom (0-9 keys, selectable list 25%-800%) and Pan (Arrow keys).
3. **Current Features:**
    - **Adjustments:** Exposure, Contrast, Highlights, Shadows, Whites, Blacks, Temperature, Tint, Vibrance, Saturation, Sharpness.
    - **Dithering:** 
        - Multi-level dither with integrated Posterization.
        - Algorithms: Threshold, Random, Bayer (2x2 to 8x8), Blue Noise, Diffusion Approx, Stucki, Atkinson, Gradient Based, Lattice-Boltzmann, Stippling.
        - **Advanced Stippling:** Anti-aliased "Ink on Paper" reconstruction with controls for Spacing, Min/Max Dot Size, and Softness.
    - **Gradient Remap:** Multi-stop system with HSB/RGB editing and interpolation.
    - **Export:** PNG, JPG, WebP with Quality/Compression control, Transparency toggle, and Resolution Scaling (Aspect ratio lock).
    - **I/O:** Drag & drop, Clipboard (Paste), and System File Picker.

## Status: v0.9 (2026-02-19)
- [x] Nearest Neighbor filtering for sharp dither clarity.
- [x] Comprehensive Dithering suite (11 algorithms).
- [x] High-quality anti-aliased Stippling algorithm.
- [x] Overhauled v0.9 UI based on Cascadia Mono design.
- [x] Tactical Keyboard UI & Context-Aware Shortcuts.
- [x] Robust Image Export system (Fixed gamma & encoders).
- [x] Image-only focus (Removed all video code).

## Recent Achievements
- Successfully transitioned to a professional v0.9 UI layout with top-aligned context-aware shortcuts and bottom-aligned zoom controls.
- Implemented a complex 3x3 neighborhood search for Stippling to eliminate grid artifacts and allow dot overlapping.
- Integrated all "Adjust" tab settings into the Stippling dot-size calculation for consistent results.
- Removed legacy video processing code and broken FFmpeg dependencies to streamline the application.
- Added selectable zoom presets (25% - 800%) and centered view resetting for [Fit] and [100%].
