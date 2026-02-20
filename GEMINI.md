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
    - Optimized Order: Exposure/WB -> Contrast/Brightness -> Highlights/Shadows -> Saturation/Vibrance -> RGB Curves -> Dithering -> Gradient Remap.
2. **Key UI Elements:**
    - Matrix Green terminal-inspired layout with monochromatic palette.
    - Tactical Keyboard-centric navigation (Hierarchical Menus with [MODE] labels).
    - Centered "Oscilloscope" style editing overlay for parameter changes.
    - High-fidelity **Per-Channel RGB Curves** editor (Master, R, G, B) with 4x4 background grid.
    - Blender-style Color Ramp for Gradient Remap with immediate GPU color updates.
    - Real-time live preview with Zoom (selectable list 25%-800%) and Pan (Arrow keys).
3. **Current Features:**
    - **Adjustments:** Exposure, Contrast, Highlights, Shadows, Whites, Blacks, Temperature, Tint, Vibrance, Saturation, Sharpness.
    - **Dithering:** 
        - Multi-level dither with integrated Posterization.
        - 10 Algorithms: Threshold, Random, Bayer (2x2 to 8x8), Blue Noise, Diffusion Approx, Stucki, Atkinson, Gradient Based, Lattice-Boltzmann.
    - **Gradient Remap:** Multi-stop system with HSB/RGB editing and interpolation.
    - **Export:** PNG, JPG, WebP with Quality/Compression control, Transparency toggle, and Resolution Scaling (Aspect ratio lock).
    - **I/O:** Drag & drop, Clipboard (Paste), and System File Picker.

## Status: v0.9 (2026-02-19)
- [x] Nearest Neighbor filtering for sharp dither clarity.
- [x] Comprehensive Dithering suite (10 algorithms).
- [x] Overhauled v0.9 UI based on Cascadia Mono design.
- [x] Tactical Keyboard UI & Context-Aware Shortcuts.
- [x] Per-channel RGB Curve editing with intelligent LUT generation.
- [x] Robust Image Export system (Fixed gamma & encoders).
- [x] Image-only focus (Removed all video code).

## Recent Achievements
- Successfully implemented **Per-Channel RGB Curves** allowing independent control over Master, Red, Green, and Blue channels.
- Improved curve interpolation logic to support linear extrapolation, ensuring X-axis movement of endpoints "crushes" or "clips" values correctly.
- Reordered the adjustment pipeline to apply global Exposure and Contrast first, fixing the bias where highlights/whites were difficult to adjust.
- Fixed Gradient Ramp to trigger immediate GPU updates when stop colors are modified.
- Cleaned up the shortcut bar and footer for a more professional, distraction-free interface.
- Streamlined the application by removing broken FFmpeg dependencies and experimental stippling code.
