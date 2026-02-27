# GEMINI.md - VibeDither Project Context

## Project Overview
**VibeDither** is a modern, futuristic, and minimalistic image and video dithering application designed for Windows 11. It features a high-contrast terminal aesthetic and provides high-performance, GPU-accelerated processing tools for artists and designers. 

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
    - Real-time live preview with Zoom (1-4:Zoom) and Pan (Arrow keys / WASD).
3. **Current Features:**
    - **Adjustments:** Exposure, Contrast, Highlights, Shadows, Whites, Blacks, Temperature, Tint, Vibrance, Saturation, Sharpness.
    - **Dithering:** 
        - Multi-level dither with integrated Posterization.
        - 10 Algorithms: Threshold, Random, Bayer (2x2 to 8x8), Blue Noise, Diffusion Approx, Stucki, Atkinson, Gradient Based, Lattice-Boltzmann.
    - **Gradient Remap:** Multi-stop system with HSB/RGB editing and interpolation.
    - **Export:** PNG, JPG, WebP, and MP4/MKV/MOV Video with Quality/Compression control.
    - **I/O:** Drag & drop, Clipboard (Paste), and System File Picker.

## Status: v1.1 Final (Video & UX Overhaul)
- [x] Tactical Bit Depth System (1-4 bit range).
- [x] Discrete Palette Editor with Box UI & Smart Interpolation.
- [x] Full Keyboard Preset Navigation (Tactical Vertical Lists).
- [x] Initial Video Support (Load first frame of video) - VERIFIED.
- [x] Real-time Video Playback (Preview) - VERIFIED.
- [x] Video Export with Dithering & Audio - VERIFIED.
- [x] Hidden FFmpeg Windows (Background processing) - NEW.
- [x] Video Export Progress Bar & Percentage - NEW.
- [x] Fixed Video Scaling Bug (2x/4x support) - NEW.
- [x] Color Accuracy Fixes (BT.709 tagging, VUI, and Range correction).
- [x] Granular Reset System (Reset Light/Color/Curves independently).
- [x] Smart Content Loading (Preserve Dither settings across media).
- [x] Context-Aware Export UI (Shows only relevant formats).
- [x] Full WASD & Enter support for all UI menus and presets.
- [x] Refined Global Shortcuts (1-4:Zoom, WASD Navigation).

## Recent Achievements
- **Stealth Video Processing:** Hidden all FFmpeg/FFprobe command prompts on Windows using `CREATE_NO_WINDOW` for a seamless "single app" feel.
- **Video Export Progress Tracking:** Implemented a real-time progress bar and percentage display in the Export window for video rendering.
- **Fixed Video Scaling:** Resolved a glitch where non-native resolutions (e.g. 2x scale) produced broken video files.
- **GPU Performance Milestone:** Verified amazing high-performance execution on legacy hardware (NVIDIA GT 730), proving the efficiency of the `wgpu` pipeline.
- **Removed Heavy Dependencies:** Replaced `ffmpeg-sidecar` with native `std::process::Command` to reduce bloat and allow low-level process control.
- **VibeDither v1.1 Milestone:** Finalized video processing pipeline with stealth execution and improved feedback.
- **Fixed Video Export Color Accuracy:** Resolved "hallucinated" colors and contrast shifts by explicitly tagging input as full-range RGBA and output as limited-range BT.709. Integrated VUI (Video Usability Information) and `-tune grain` for optimal dither preservation.
- **Context-Aware Export UI:** Implemented logic to hide/show export formats based on content. Video files now default to Video export, while images show PNG/JPG/WebP.
- **Granular Reset Logic:** Added dedicated reset buttons for Light and Color sections. These now only reset their specific parameters, preserving Dither and Palette settings.
- **Smart Media Switching:** Updated the loading pipeline to reset basic adjustments (Light/Color/Curves) but preserve the complex Dither and Palette setup when switching between different images or videos.
- **Universal WASD & Enter Support:** Standardized keyboard navigation across all menus including Adjust Presets, Palette Presets, Bayer Size, and Export. Added `Enter` as a standard selection key.
- **Optimized Zoom Workflow:** Redefined number keys `1-4` for faster preview control (Zoom Out, Zoom In, 100%, and Fit to Screen).
- **Enhanced Palette Preset Previews:** Integrated color bar previews directly into the keyboard-driven Palette Preset menu for immediate visual feedback during selection.
- **Initial Video Support:** Implemented video frame extraction using `ffmpeg-sidecar`. Users can now load video files (.mp4, .mkv, .mov, etc.), and the application will extract and display the first frame for editing using the existing image pipeline.
- **Unified "Load Content" Interface:** Renamed the load button and updated file dialogs to support both images and videos seamlessly.
- **VibeDither v1.1 Final Version:** Reached full stability for image processing with a robust, keyboard-centric workflow.
- **Implemented Tactical Bit Depth:** Replaced generic "Posterize" with a dedicated Bit Depth system (1-4 bits), providing precise control over color levels ($2^n$).
- **Enhanced Palette Editor:** Transformed the gradient ramp into a discrete box-based Palette Editor with intelligent color interpolation on bit depth changes.
- **Keyboard-Centric Presets:** Implemented tactical vertical lists for Adjust and Palette presets, enabling full navigation and application via keyboard (P to open, Arrows to select, Space to apply).
- **Expanded Shortcuts:** Integrated system-level shortcuts for productivity, including CTRL+O for file picking and CTRL+V for immediate clipboard image pasting.
- **Polished UI Contrast:** Refined hover and active states to use high-contrast dark green text on bright green backgrounds, ensuring legibility in the terminal aesthetic.
- **Robust Navigation Toggles:** Added menu toggling logic for Bits (B), Palette (G), and Presets (P) to streamline the editing workflow.
- **VibeDither v1.0 Milestone:** Finalized the application for release with a polished UI and robust static image processing pipeline.
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
