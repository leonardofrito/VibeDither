i am working on a design project that features modern, futuristic, minimalistic and terminal like graphics, and i found this app that is supposed to dither images, but the app is bad. Lets make our own. For now, your work is not to code, but to help me write the best Chat GPT prompt so you write the exact application i need. Ask me anything you need, any information, any decision, lets leave nothing to guessing. Make a list of things we need to figure out and lets fill it up, suggest both libraries to use, coding language, form of execution, etc. For the first version we will only worry about suppporting .jpg, .png .avif .webp and other image formats, and also support pasting the image from the clipboard. The app, must be for windows 11, it dithers image. It must feature different styles of dithering, different patterns. It must support both color and black and white. It must have basic image manipulation controls that affect the image in real time (ie: Brightness, Exposure, Contrast, Shadows, Highlights, RGB Curves, Saturation, Vibrance, HSL Slider, Color Mixer). I must also be able to have controls over the dithering, for example, It's scale. At the end, add a gradient remap option, in case i want to change the colors, make the gradient remap work like the Blender Gradient Ramp node, where i start with 2 nodes but i can add more, and add options to the gradient ramp that change the way the color is remaped (ie Perceptual, Smooth, Linear). Add a option to zoom in on the canvas to see smaller details, and a dropdown in the bottom to change zoom % and reset the view to 100% or to Fit on Screen. Exporting to png, jpg, avif, webp or copy to clipboard. As i know some effects will take time depending on image size and other factors, lets add a simple line across the top bar that distorts while the image is being processed as a visual information that the program is working, and block the person from making more changes to the image. Also add a button to toggle live preview or not. Its a lot of stuff so we are not doing it all on one single prompt, lets break it down on multiple prompts and work step by step

/End User/
---
/Start GPT/
---
Understood. We will treat this as a **specification-extraction phase**, not coding, and the goal is to end with a **single, extremely precise “master prompt”** (or a small sequence of prompts) that can later be used to generate the application with minimal ambiguity.

Below is a **structured checklist of decisions and unknowns** we must resolve. For each section, I explain why it matters and then list **concrete questions** you should answer. We will fill this step by step and only then synthesize the final ChatGPT prompts.

---

## 1. Product Definition (Non-negotiable foundation)

We need to lock what this app _is_ and _is not_.

Questions:

1. Is this a **desktop-native application** or a **desktop app using web technologies**?
    
    - Native (C++ / Rust / C#)
        
    - Web stack (Electron, Tauri, Wails)
        
2. Is this intended to be:
    
    - A **personal tool**
        
    - An **open-source project**
        
    - A **commercial product**
        
3. Is offline-only acceptable, or must it be **fully offline by design** (no telemetry, no network access)?
    
4. Target audience:
    
    - Designers
        
    - Artists
        
    - Technical users
        
    - General users
        

These answers affect framework choice, UI complexity, and performance tradeoffs.

---

## 2. Platform & Distribution (Windows 11 specifics)

You stated Windows 11 explicitly; we need to define how “Windows-native” it must feel.

Questions:

1. Packaging format:
    
    - `.exe` installer
        
    - Portable `.exe`
        
    - Microsoft Store compatible (MSIX)
        
2. Windows integration requirements:
    
    - Drag & drop from Explorer?
        
    - “Paste image from clipboard” via Ctrl+V anywhere?
        
3. GPU usage:
    
    - Optional GPU acceleration
        
    - Required GPU acceleration
        
    - CPU-only fallback mandatory?
        
4. Minimum supported hardware:
    
    - Integrated GPU acceptable?
        
    - High-resolution images (e.g., 8K+) expected?
        

---

## 3. Technology Stack (Language + Framework)

This is one of the most important decisions.

### Candidate directions (to choose from)

- **C++ + Qt / Skia**
    
- **Rust + wgpu + egui / iced**
    
- **C# + WPF / WinUI 3**
    
- **Tauri (Rust backend) + Web UI**
    
- **Electron + WebGL/WebGPU**
    
- **Python is likely NOT appropriate** for this scope/performance
    

Questions:

1. Do you prioritize:
    
    - Maximum performance
        
    - Developer ergonomics
        
    - Visual polish
        
2. Are you comfortable with:
    
    - Rust
        
    - C++
        
    - C#
        
    - Web stack (TypeScript, React, Svelte)?
        
3. Is WebGPU acceptable, or do you want to avoid browser-style rendering entirely?
    
4. Do you want shader-based effects (GPU dithering), or CPU-based image processing is acceptable?
    

---

## 4. UI / UX Philosophy (Terminal-like but modern)

You described: _modern, futuristic, minimalistic, terminal-like_.

We need to define this concretely.

Questions:

1. Color scheme:
    
    - Monochrome green-on-black?
        
    - Custom accent color?
        
    - User-selectable themes?
        
2. Typography:
    
    - Monospace only?
        
    - Mixed (UI sans + monospace for labels)?
        
3. Layout:
    
    - Left control panel + main canvas (like the reference)?
        
    - Dockable panels?
        
    - Fixed layout only?
        
4. Animation philosophy:
    
    - Subtle (micro-interactions only)
        
    - Visible glitch/scanline effects
        
    - No animation except processing indicator
        
5. Accessibility:
    
    - Keyboard navigation required?
        
    - High-contrast mode?
        

---

## 5. Image Input Pipeline

You already defined supported formats; we must formalize behavior.

Supported:

- JPG
    
- PNG
    
- AVIF
    
- WebP
    
- Others (TIFF? BMP? EXR?)
    

Questions:

1. Maximum image size (soft limit)?
    
2. Color space handling:
    
    - sRGB only?
        
    - Preserve embedded ICC profiles?
        
3. Alpha channel:
    
    - Preserve transparency?
        
    - Option to dither alpha?
        
4. Clipboard behavior:
    
    - Auto-detect image vs bitmap?
        
    - Paste replaces current image or creates new session?
        
5. Drag & drop behavior:
    
    - Replace current image?
        
    - Ask for confirmation?
        

---

## 6. Image Processing Stack (Pre-Dither Controls)

You listed a **very full color pipeline**. We need to define order and behavior.

Controls you want:

- Brightness
    
- Exposure
    
- Contrast
    
- Shadows
    
- Highlights
    
- RGB Curves
    
- Saturation
    
- Vibrance
    
- HSL
    
- Color Mixer
    

Questions:

1. Processing order:
    
    - Fixed order (Photoshop-like)?
        
    - User-reorderable stack?
        
2. Bit depth:
    
    - 8-bit internal?
        
    - 16-bit float internal?
        
3. Are adjustments:
    
    - Destructive?
        
    - Non-destructive with live recompute?
        
4. Should toggling live preview:
    
    - Disable recomputation?
        
    - Cache last processed image?
        
5. Reset options:
    
    - Per-control reset?
        
    - Global reset?
        

---

## 7. Dithering System (Core Feature)

This is the heart of the app and must be specified precisely.

### Dithering types (to confirm)

Examples:

- Floyd–Steinberg
    
- Bayer (ordered)
    
- Atkinson
    
- Jarvis–Judice–Ninke
    
- Blue noise
    
- Error diffusion variants
    
- Custom matrix upload?
    

Questions:

1. Black & white modes:
    
    - Threshold-based
        
    - Adaptive threshold
        
2. Color dithering:
    
    - Per-channel
        
    - Luminance-based
        
    - Indexed palette?
        
3. Palette control:
    
    - Fixed palettes?
        
    - User-defined palettes?
        
4. Scale parameter:
    
    - Pixel size?
        
    - Pattern frequency?
        
5. Preview:
    
    - Real-time for all algorithms?
        
    - Progressive refinement for slow ones?
        

---

## 8. Gradient Remap (Blender-like)

You referenced Blender’s Gradient Ramp node. We must define parity level.

Questions:

1. Gradient space:
    
    - Luminance-based
        
    - Value-based
        
    - User-selectable input?
        
2. Color interpolation modes:
    
    - Linear
        
    - Smooth
        
    - Perceptual (OKLab / LAB)?
        
3. Node interaction:
    
    - Drag to reposition?
        
    - Numeric input?
        
4. Can gradient be saved/loaded as presets?
    
5. Does gradient remap occur:
    
    - Before dithering?
        
    - After dithering?
        
    - Toggleable?
        

---

## 9. Canvas Interaction

Questions:

1. Zoom:
    
    - Mouse wheel?
        
    - Ctrl + wheel?
        
2. Pan:
    
    - Click + drag?
        
    - Spacebar + drag?
        
3. Zoom presets:
    
    - 25%, 50%, 100%, 200%, Fit
        
4. Pixel grid overlay?
    
5. Nearest-neighbor zoom vs smooth zoom?
    

---

## 10. Processing State & Feedback

You described a **distorting line across the top bar**.

Questions:

1. Visual style:
    
    - CRT scanline?
        
    - Wave distortion?
        
    - Noise bar?
        
2. Should processing:
    
    - Block all UI?
        
    - Allow cancel?
        
3. Should heavy operations:
    
    - Run async?
        
    - Be cancellable?
        
4. Status text or visual-only?
    

---

## 11. Export & Output

Questions:

1. Export formats:
    
    - PNG, JPG, AVIF, WebP (confirmed)
        
2. Export options:
    
    - Quality slider?
        
    - Chroma subsampling?
        
3. Metadata:
    
    - Strip EXIF?
        
    - Preserve EXIF?
        
4. Copy to clipboard:
    
    - As bitmap?
        
    - As PNG buffer?
        

---

## 12. Project Scope Control (Very Important)

To avoid overengineering:

Questions:

1. Presets system in v1?
    
2. Undo/redo stack in v1?
    
3. Batch processing:
    
    - Out of scope for now?
        
4. Command-line interface:
    
    - Out of scope?
        

---

## Next Step (What I need from you now)

Please answer **Sections 1–3 first** (Product Definition, Platform, Technology Stack).  
Once those are locked, I will:

1. Propose a **recommended tech stack** (with rationale).
    
2. Draft **Prompt #1**: “You are building a Windows 11 dithering application…” covering architecture.
    
3. Then we will iteratively refine:
    
    - UI prompt
        
    - Image pipeline prompt
        
    - Dithering algorithms prompt
        
    - Gradient ramp prompt
        

No code yet. Only precision.

/End GPT/
---
/Start User/
---

It is a desktop native app, for an open source project, target towards designers, artists and general users. Lets package to a portable .exe so its easier to share and download. Both drag & drop, copy past from anywhere, and click to upload are perfect. Optional gpu acceleration for faster processing, lets make the first version gpu only with intent of making a cpu version later. I dont expect 8k resolution, but users might, so i wont go out of my way to trade off stuff to support 8k, but if i dont have to trade i will accept it. As for language, i dont know any. I prioritize performance. Dont use any language, so any language is the same. Lets do a terminal like aproach simmilar to the screenshot, on the left the menu with the settings, theres a button to flip it to the right side. Users can change the accent color and background color, monospace font that can be changed by the user. Fixed Layout only. |No animations other than the processing indicator. It is already a high contrast mode, and we can add keyboard navigation later. No soft limit, users deal with it, convert it sRGB so we dont have to worry about color profiles. Yes, preserve transparency or dither alpha. Ask if it creates a new session. Auto detect Image from clipboard. Processing order fixed like photoshop. Does 16 bit float make a difference? Non destructive with live recompute yes. And live preview should toggle recomputation. Per Control reset via a single reset button on the side. Ditherign types, thresshold based for B&W. Color dithering all 3 are nice options. Palette control will offer some default ones and options to have user made ones. Preview realtime for all yes. Gradient ramp can be luminance or value based, the user selects. Color interpolation is correct yes, user chooses. Node interaction is by drag to reposition. Gradients can be saved, the app will save on a folder where it saves custom user related options and presets. Gradient remap as a toggleable option is amazing. Canvas interaction of zooming by ctrl + wheel is nice, click + drag to pan, and zoom presets (1%, 2%, 5%, 10%, 25%, 50%, 100%, 200%, 400%, 800%, Fit). No pixel grid overlay, nearest neighbor zoom. A CRT Scanline is cool for the processing state. Processing blocks the ui and allow cancelling. Heavy operations at async are better or worse? yes they should be cancellable. Add quality sliders for the export, i dont know whats chroma subsampling. Add the option to preserve exif or strip it. Copy to clipboard as bitmap or png buffer yes. Only the undo redo for v1. Presets will be done later after we finish the ability to change things and etc. Batch processing is cool and can be added later down the line too, cli is a bit overkill i think

/End User/
---
/Start GPT/
---

Good. That locks enough to move forward cleanly. I will do three things in this response:

1. Resolve the **open technical questions you explicitly asked** (e.g., 16-bit float, async).
    
2. Propose a **recommended technology stack** that fits _all_ constraints.
    
3. Define **Phase-1 prompt structure** and give you the **first concrete prompt** we will later refine.
    

No code yet.

---

## 1. Clarifications & Decisions (Resolved)

### 16-bit float: does it matter?

Yes, materially.

Using **16-bit floating point internally** (half-float) is important because:

- You are stacking many color operations (curves, HSL, vibrance, exposure).
    
- Dithering amplifies quantization errors.
    
- Gradient remapping benefits from smooth precision before reduction.
    

Decision:

- **Internal pipeline: 16-bit float per channel**
    
- **Output: format-dependent (8-bit for PNG/JPG, higher if supported)**
    

This is standard in professional tools and aligns with your quality goals.

---

### Async processing: better or worse?

Better, and required.

Reason:

- GPU compute will still stall frames if run synchronously.
    
- Async allows:
    
    - Cancel mid-process
        
    - UI feedback (CRT scanline indicator)
        
    - Clean “live preview OFF” behavior
        

Decision:

- **All processing runs asynchronously**
    
- **UI hard-locked during processing**
    
- **Cancel button interrupts GPU job safely**
    

---

### Chroma subsampling (brief, no decision needed yet)

This only affects JPG/WebP.

- 4:4:4 = full color fidelity
    
- 4:2:0 = smaller file, lower chroma resolution
    

We will:

- Expose it as an **advanced export option later**
    
- Not include in v1 UI unless you want
    

For now: **quality slider only** is acceptable.

---

## 2. Recommended Technology Stack (Based on Your Constraints)

You said:

- Performance first
    
- GPU-only v1
    
- Windows 11 native feel
    
- Open source
    
- No language preference
    
- Terminal-like custom UI
    
- Image-heavy, shader-friendly
    

### Strong Recommendation

**Rust + wgpu (WebGPU) + egui**

#### Why this stack fits almost perfectly

**Rust**

- High performance
    
- Memory-safe (important for image buffers)
    
- Excellent for open-source longevity
    

**wgpu (WebGPU abstraction)**

- Native GPU compute (DirectX 12 on Windows)
    
- Excellent for:
    
    - Dithering
        
    - Gradient remapping
        
    - Color transforms
        
- Future-proof (CPU fallback possible later)
    

**egui**

- Immediate-mode UI
    
- Very good for:
    
    - Fixed layouts
        
    - Terminal-style UI
        
    - Custom widgets (gradient ramp, scanline bar)
        
- No HTML, no browser baggage
    
- Portable `.exe` friendly
    

This avoids:

- Electron overhead
    
- Qt licensing concerns
    
- WinUI styling constraints
    

This is the same class of stack used by modern creative tools that care about performance and control.

Decision (unless you object):

- **Language:** Rust
    
- **Rendering & compute:** wgpu
    
- **UI:** egui
    
- **Image decoding:** image + ravif + libwebp bindings
    
- **Clipboard:** Windows API via Rust bindings
    

If you want an alternative later, the main competitor would be **C++ + Skia + DirectX**, but complexity is significantly higher.

---

## 3. How We Will Break the Prompts (Roadmap)

We will **not** write one giant prompt. That leads to hallucinated architecture.

Instead, we will produce **a sequence of prompts**, each with a single responsibility.

### Planned Prompt Sequence

**Prompt 1 — Application Architecture**

- Tech stack
    
- App structure
    
- GPU pipeline overview
    
- Async processing model
    
- State management
    
- File I/O and clipboard
    

**Prompt 2 — UI & Interaction Specification**

- Fixed layout
    
- Terminal aesthetic
    
- Left/right panel toggle
    
- Canvas behavior
    
- Zoom & pan
    
- Processing indicator behavior
    

**Prompt 3 — Image Processing Pipeline**

- Exact processing order
    
- 16-bit float handling
    
- Live preview toggle logic
    
- Non-destructive recompute
    
- Undo/redo model
    

**Prompt 4 — Dithering System**

- Algorithms
    
- Parameters
    
- Color vs B&W modes
    
- Palette system
    

**Prompt 5 — Gradient Ramp System**

- Node model
    
- Interpolation math
    
- UI interaction
    
- Save/load format
    

**Prompt 6 — Export & Clipboard**

- Formats
    
- Quality handling
    
- EXIF options
    
- Clipboard formats
    

Each prompt builds on the previous one.

---

## 4. Prompt #1 — Architecture (Draft)

This is the **first real prompt** you will later paste into ChatGPT to generate code.  
We will refine it, but this is the baseline.

---

### PROMPT 1 — APPLICATION ARCHITECTURE

> You are a senior systems engineer and GPU programmer.
> 
> Design a Windows 11 desktop-native, open-source image dithering application focused on performance and real-time GPU processing.
> 
> **Constraints & Goals**
> 
> - Platform: Windows 11
>     
> - Distribution: portable `.exe`
>     
> - GPU-only processing (CPU fallback explicitly out of scope for v1)
>     
> - Target users: designers, artists, general users
>     
> - Offline-only, no telemetry
>     
> - Fixed-layout UI with a terminal-like aesthetic
>     
> 
> **Technology Stack (mandatory)**
> 
> - Language: Rust
>     
> - GPU: wgpu (DirectX 12 backend)
>     
> - UI: egui
>     
> - Image decoding: JPG, PNG, AVIF, WebP
>     
> - Clipboard support: paste image from anywhere
>     
> 
> **Core Architectural Requirements**
> 
> 1. All image processing runs asynchronously on the GPU.
>     
> 2. Internal image representation uses 16-bit floating point per channel.
>     
> 3. Processing is non-destructive and recomputed from source on each change.
>     
> 4. UI is fully blocked during processing, with a cancelable job system.
>     
> 5. A visual processing indicator appears as a CRT-style scanline distortion across the top bar while processing.
>     
> 
> **Input Methods**
> 
> - Drag & drop image
>     
> - File picker
>     
> - Clipboard paste (auto-detect image)
>     
> - Prompt user whether loading creates a new session
>     
> 
> **Output**
> 
> - PNG, JPG, AVIF, WebP
>     
> - Copy to clipboard (bitmap or PNG buffer)
>     
> 
> Describe:
> 
> - High-level app architecture
>     
> - GPU processing pipeline
>     
> - State management model
>     
> - Async job and cancellation model
>     
> - How UI state and processing state interact
>     
> 
> Do not write code yet. Focus on structure, responsibilities, and data flow.

---

## Next Step

Confirm or adjust **the technology stack choice**.

Once confirmed, I will:

- Lock Prompt 1
    
- Move to **Prompt 2: UI & Interaction Specification**
    
- Start defining exact widget behavior (including the gradient ramp editor UI)
    

If you want to change the stack now, this is the moment.

/End GPT/
---
/Start User/
---

I don't have GPT Codex, but I have access to Gemini 3 on my CLI, so let's make a Gemini 3 prompt so he starts building the base, the structure to work on from there. Let's start vibe coding. Using all the information i gave you, build the master prompt for the first phase of the creation of the program, list all the dependecies and programs were gonna use, list useful docs or ask me to research them. Lets pack it up and hit the road

/End User/
---
/Start GPT/
---

Below is a **single, self-contained master prompt** designed specifically for **Gemini 3 running in a CLI**, optimized for _vibe coding_ while still forcing a correct, scalable foundation.

This is **Phase 1 only**: bootstrap, architecture, scaffolding, and first visible pixels on screen.  
No feature creep, no half-implemented systems.

You can paste this verbatim into Gemini 3.

---

### MASTER PROMPT — PHASE 1

**Project: GPU-based Image Dithering App (Windows 11)**

You are a senior systems engineer, GPU programmer, and creative-tool architect.

Your task is to **bootstrap the foundation of a Windows 11 desktop-native, open-source image dithering application**, optimized for performance and future extensibility.

This is **Phase 1**. The goal is to establish:

- Correct project structure
    
- Working window + UI
    
- GPU pipeline skeleton
    
- Async job system
    
- Image loading (no full processing yet)
    
- Clear extension points for later phases
    

Do **not** attempt to implement every feature yet.

---

## 1. High-Level Product Definition

- Desktop-native application
    
- Platform: **Windows 11**
    
- Distribution: **portable `.exe`**
    
- Open source
    
- Offline-only, no telemetry
    
- Target users: designers, artists, general users
    
- GPU-only processing for v1 (CPU fallback explicitly deferred)
    

---

## 2. Technology Stack (MANDATORY)

You must use the following stack:

- **Language:** Rust (stable)
    
- **GPU abstraction:** `wgpu` (DirectX 12 backend on Windows)
    
- **UI framework:** `egui`
    
- **Windowing:** `winit`
    
- **Async:** `tokio` or equivalent
    
- **Image decoding:**
    
    - `image` crate
        
    - AVIF support via `ravif` or equivalent
        
    - WebP support via bindings
        
- **Clipboard:** Windows clipboard via Rust bindings
    
- **Build system:** `cargo`
    

Do **not** use:

- Electron
    
- Qt
    
- Web UI
    
- Python
    

---

## 3. Architectural Principles (Non-negotiable)

- Internal image representation: **16-bit float per channel**
    
- Non-destructive processing model
    
- All image processing runs **asynchronously**
    
- UI is **blocked during processing**
    
- Processing jobs are **cancelable**
    
- GPU compute is the primary execution path
    
- Fixed-layout UI only (no docking)
    

---

## 4. Phase 1 Scope (What You SHOULD Build)

### 4.1 Project Structure

Create a clean, extensible Rust project layout with:

- Clear separation between:
    
    - UI
        
    - GPU pipeline
        
    - Image I/O
        
    - App state
        
    - Async job system
        

Document why each module exists.

---

### 4.2 Window + UI Skeleton

- Create a window using `winit`
    
- Integrate `egui`
    
- Fixed layout:
    
    - Left-side control panel (placeholder widgets)
        
    - Main canvas area (empty or placeholder texture)
        
- Terminal-like aesthetic:
    
    - Dark background
        
    - High-contrast accent color (configurable later)
        
    - Monospace font (configurable later)
        
- No animations except placeholders for later
    

---

### 4.3 GPU Pipeline Skeleton

- Initialize `wgpu`
    
- Create:
    
    - Device
        
    - Queue
        
    - Surface
        
- Set up:
    
    - A basic render pass
        
    - A placeholder compute pipeline (no real processing yet)
        
- Ensure the pipeline can later accept:
    
    - Input textures
        
    - Output textures
        
    - Parameters via uniform buffers
        

---

### 4.4 Async Job System

- Implement a job manager that:
    
    - Runs GPU tasks asynchronously
        
    - Blocks UI interaction while running
        
    - Supports cancellation
        
- No real image processing yet—use a simulated GPU workload if needed
    
- Expose job state to UI (idle / processing / canceled)
    

---

### 4.5 Image Input (Minimal)

- Implement:
    
    - Load image via file picker
        
    - Drag & drop image into window
        
    - Paste image from clipboard (auto-detect)
        
- Convert all images to:
    
    - sRGB
        
    - 16-bit float internal format
        
- Ask user (via modal or console log for now):
    
    - “Create new session or replace current image?”
        

No dithering yet—just confirm loading and display a placeholder.

---

### 4.6 Canvas Interaction (Minimal)

- Implement:
    
    - Ctrl + mouse wheel zoom
        
    - Click + drag pan
        
    - Nearest-neighbor scaling
        
- Zoom presets can be stubbed (not functional yet)
    

---

## 5. Explicitly Out of Scope for Phase 1

Do NOT implement yet:

- Dithering algorithms
    
- Color controls
    
- Gradient ramp
    
- Presets
    
- Batch processing
    
- CLI mode
    
- Export logic
    
- Undo/redo (only plan for it)
    

---

## 6. Deliverables

You must output:

1. **Project folder structure**
    
2. **Dependency list (`Cargo.toml`)**
    
3. **Module responsibilities**
    
4. **High-level data flow diagram (textual)**
    
5. **Explanation of GPU + async architecture**
    
6. **Minimal runnable scaffold code**
    
    - App launches
        
    - Window opens
        
    - UI renders
        
    - Image can be loaded
        
7. **Clear TODO markers for Phase 2**
    

Do **not** over-polish.  
Do **not** invent missing features.  
Do **not** skip explanations.

---

## 7. Required Documentation References

List and briefly explain why each is relevant:

- `wgpu` official documentation
    
- `egui` official documentation
    
- `winit` documentation
    
- Windows clipboard API references for Rust
    
- AVIF/WebP decoding references
    

If something requires external research, explicitly say:

> “User should research X before Phase 2”

---

## 8. Tone & Style

- Engineer-to-engineer
    
- Precise
    
- No marketing language
    
- No emojis
    
- No filler
    

---

## 9. End Condition

End your response with:

- A **Phase 2 readiness checklist**
    
- A list of **architectural decisions that must not change**
    

---

That’s it.  
This prompt is intentionally strict—**do not soften it**.

Once Gemini finishes Phase 1, we will layer:

- UI polish
    
- Processing stack
    
- Dithering
    
- Gradient ramp
    
- Export
    

You are now cleared to start building the foundation.