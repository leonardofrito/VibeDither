# GEMINI.md - VibeDither Project Context

## Project Overview
**VibeDither** is a modern, futuristic, and minimalistic image and video dithering application designed for Windows 11. It features a high-contrast terminal aesthetic and provides high-performance, GPU-accelerated processing tools for artists and designers. 

**Note:** The application currently supports static image processing and initial video frame extraction for dithering. Full video playback and export are planned for v1.2.

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

## Status: v1.2 Development (Video Support)
- [x] Tactical Bit Depth System (1-4 bit range).
- [x] Discrete Palette Editor with Box UI & Smart Interpolation.
- [x] Full Keyboard Preset Navigation (Tactical Vertical Lists).
- [x] Initial Video Support (Load first frame of video) - VERIFIED.
- [ ] Real-time Video Playback (Preview).
- [ ] Video Export with Dithering.
- [x] Official v1.1 Final Release.
- [x] High-contrast Matrix Green UI with dark-green contrast text on hover.
- [x] CTRL+O (Open) and CTRL+V (Paste) shortcut support.
- [x] Comprehensive Dithering suite (10 algorithms).

## Recent Achievements
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

## Video Support Research (v1.2 Planning)

### 1. FFmpeg Reference Links
- [Rust FFmpeg (ffmpeg-next) Documentation](https://docs.rs/rust_ffmpeg/latest/rust_ffmpeg/index.html)
- [Official FFmpeg Documentation](https://ffmpeg.org/documentation.html)
- [FFmpeg Command Line Tool Documentation](https://ffmpeg.org/ffmpeg.html)

### 2. Core Architecture for Video
To reintegrate video support while maintaining the `wgpu` pipeline, the following components are required:
- **Decoder (`ffmpeg-next`):** Opens the container (`avformat`), identifies the video stream, and uses the appropriate codec (`avcodec`) to extract raw frames.
- **Conversion (`libswscale`):** Most videos are in YUV format. These must be converted to `Rgba8UnormSrgb` (matching the current `Pipeline` input) on the CPU before being uploaded to the GPU.
- **GPU Pipeline Integration:** For each decoded frame, `queue.write_texture` is called to update the `input_texture`. The `Pipeline::render` function then processes the frame using the existing shaders (Adjustments + Dithering).
- **Encoder (`ffmpeg-next`):** Processed frames are read back from the GPU (`read_back_image` logic) and sent to an encoder to be compressed and saved into a new container.

### 2. Key Components & Libraries
- **`ffmpeg-next`:** The modern Rust bindings for FFmpeg C libraries. Essential for high-performance, in-memory frame access. Recommended over `rust_ffmpeg` (which is primarily a CLI command builder) to enable seamless GPU integration and live previews.
    - `ffmpeg::format::input`: For opening video files.
    - `ffmpeg::decoder::Video`: For decoding packets into frames.
    - `ffmpeg::software::scaler`: For YUV -> RGBA conversion.
    - `ffmpeg::encoder::Video`: For compressing processed frames.
    - `ffmpeg::format::output`: For muxing streams into a file (MP4, MKV, etc.).
- **`wgpu` Buffers:** Efficient read-back from the GPU is critical for export. Utilizing a buffer pool or persistent mapping can minimize the bottleneck of `copy_texture_to_buffer`.

### 3. Workflow Implementation
#### A. Importing (Decoding)
1. Initialize FFmpeg and open the input file.
2. Locate the best video stream and its decoder parameters.
3. In the UI loop or a dedicated background thread:
    - Read the next packet.
    - Decode packet into a `Frame`.
    - Scale/Convert `Frame` to RGBA8.
    - Update `wgpu` texture and trigger a redraw.

#### B. Modifying (Processing)
- The video frame simply replaces the static `input_texture`.
- All current UI sliders (Exposure, Contrast, Dither Type, Palette) remain functional and apply to the video frame in real-time.
- **Playback Sync:** Use `std::time::Instant` and the frame's PTS (Presentation Time Stamp) to match the video's native FPS.

#### C. Exporting (Encoding)
1. Setup a muxer for the target format.
2. For every frame in the source video:
    - Decode and process via `wgpu`.
    - Read back pixels to CPU.
    - Encode processed pixels into a new video packet.
    - Write packet to output file.
3. **Audio Handling:** Use stream copying (`codec_type == AVMEDIA_TYPE_AUDIO`) to preserve the original audio without re-encoding, if desired.

### 4. Technical Challenges
- **Read-back Latency:** Reading frames back from the GPU to the CPU is the slowest part of the export.
- **FFmpeg Dependencies:** Requires shared libraries (`avcodec-60.dll`, etc.) to be present on Windows, which complicates distribution.
- **Multi-threading:** Decoding and Rendering should ideally happen on separate threads to avoid UI stutters.
- **Bit Depth / Color Space:** Ensuring gamma correction (sRGB) is handled consistently between FFmpeg and `wgpu`.
