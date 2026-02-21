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

## Status: v1.0 (2026-02-21)
- [x] Official v1.0 Release.
- [x] High-contrast Matrix Green UI with full-green button highlighting.
- [x] Fixed all compilation errors and balanced implementation blocks.
- [x] Nearest Neighbor filtering for sharp dither clarity.
- [x] Comprehensive Dithering suite (10 algorithms).
- [x] Tactical Keyboard UI & Context-Aware Shortcuts.
- [x] Per-channel RGB Curve editing with intelligent LUT generation.
- [x] Robust Image Export system (Fixed gamma & encoders).

## Recent Achievements
- **VibeDither v1.0 Milestone:** Finalized the application for release with a polished UI and robust static image processing pipeline.
- **Enhanced UI Highlighting:** Implemented high-contrast green backgrounds for hovered and active buttons, ensuring a tactical terminal aesthetic.
- **Fixed Core Pipeline Delimiters:** Resolved critical compilation errors by correctly structuring the `read_back_image` function and balancing implementation blocks.
- **Implemented Adjust and Gradient Presets:** Added functionality to save, load, overwrite, and remove custom presets for image adjustments (Light, Color, Curves) and gradient ramps, including intelligent positioning and color averaging for new gradient stops.
- **Slick Minimalist UI Overhaul:** Successfully implemented a high-performance, modern slick UI using `egui` after exploring TUI (`ratatui`) and `vizia` alternatives.
- **Enhanced Tactical Styling:** Enforced a strict Matrix Green on Pure Black aesthetic with 0px rounding, 1:1 square slider handles, and frameless tactical buttons for a futuristic feel.
- **Improved Slider Interaction:** Integrated `trailing_fill` and set slider tracks to 10% brightness gray to ensure visibility and intuitive feedback on a black background.
- **Streamlined Workflow:** Removed redundant symbols and cleaned up algorithm labels to create a distraction-free editing environment.
- **Unified Preview Canvas:** Maintained a high-fidelity GPU-accelerated preview with integrated zoom/pan and context-aware keyboard shortcuts.
- **Clipboard Integration:** Implemented a functional `[ Copy ]` button for gradient stops, allowing users to quickly grab RGB values.
- **Fixed Export Color Accuracy:** Forced the internal and export pipelines to `Rgba8UnormSrgb` to ensure correct linear-to-sRGB gamma conversion during export, resolving the "dark color" shift.
- **Enhanced Readback Logic:** Streamlined `read_back_image` to support dynamic texture targets and simplified pixel retrieval.
- **Vibe Coding Milestone:** Documented the project's status as a 100% AI-vibecoded experiment (Gemini 2.5/3 CLI) in a comprehensive README.
- Successfully implemented **Per-Channel RGB Curves** allowing independent control over Master, Red, Green, and Blue channels.
- Improved curve interpolation logic to support linear extrapolation, ensuring X-axis movement of endpoints "crushes" or "clips" values correctly.
- Reordered the adjustment pipeline to apply global Exposure and Contrast first, fixing the bias where highlights/whites were difficult to adjust.
- Fixed Gradient Ramp to trigger immediate GPU updates when stop colors are modified.
- Cleaned up the shortcut bar and footer for a more professional, distraction-free interface.
- Streamlined the application by removing broken FFmpeg dependencies and experimental stippling code.
