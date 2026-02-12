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
    - Fixed order: Basic Adjustments -> Dithering (with internal Posterization) -> Gradient Remap.
2. **Key UI Elements:**
    - Matrix Green terminal-inspired layout with monospace fonts.
    - Tactical Keyboard-centric navigation (Hierarchical Menus).
    - Centered "Oscilloscope" style editing overlay.
    - Blender-style Color Ramp for Gradient Remap.
    - Real-time live preview with Zoom (0-9 keys) and Pan (Arrow keys).
3. **Current Features:**
    - **Adjustments:** Exposure, Contrast, Highlights, Shadows, Whites, Blacks, Temperature, Tint, Vibrance, Saturation, Sharpness.
    - **Dithering:** 
        - Multi-level dither with integrated Posterization.
        - Algorithms: Threshold, Random, Bayer (2x2 to 8x8), Blue Noise, Diffusion Approx, Stucki, Atkinson, Gradient Based, Lattice-Boltzmann.
        - Color Dithering toggle for all modes (except Threshold).
    - **Gradient Remap:** Multi-stop system with HSB editing support.
    - **Export:** PNG, JPG, WebP with Quality/Compression control, Transparency toggle, and Resolution Scaling (Aspect ratio lock).
    - **I/O:** Drag & drop, Clipboard (Paste), and System File Picker.

## Status: v0.3 (2026-02-12)
- [x] Nearest Neighbor filtering for sharp dither clarity.
- [x] Comprehensive Dithering suite (10 algorithms).
- [x] Tactical Keyboard UI & Shortcuts.
- [x] Blender-style Color Ramp (Gradient Remap).
- [x] Robust Image Export system (Fixed gamma & encoders).
- [ ] Support videos of all types.

## Recent Achievements
- Implemented a complete hierarchical keyboard control system for "tactic-feel" image editing.
- Fixed sRGB export color space issues by applying manual gamma correction (2.4) during readback.
- Added high-quality dither approximations optimized for GPU (Atkinson, Lattice-Boltzmann, etc.).
- Created a robust export modal with AWSD keyboard navigation and grid focus.
