# Phase 4: Video Detection & OCR Implementation Plan

**Status**: Research & Planning Complete
**Last Updated**: 2025-12-14
**Target**: Automated event detection from Tarkov gameplay footage

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture Decision](#architecture-decision)
3. [Hardware Setup](#hardware-setup)
4. [Video Capture Strategy](#video-capture-strategy)
5. [Framework & Models](#framework--models)
6. [Training Pipeline](#training-pipeline)
7. [Detection Service Design](#detection-service-design)
8. [Testing & Validation](#testing--validation)
9. [Integration with Rust App](#integration-with-rust-app)
10. [Implementation Timeline](#implementation-timeline)
11. [References](#references)

---

## Overview

### Goal
Automate detection of Tarkov game events from live gameplay footage to replace manual Stream Deck input (Phase 2b).

### Events to Detect
- Queue screen/timer
- Match found notification
- Kill notifications (with PMC/SCAV type, weapon, headshot)
- Death screen
- Extraction success
- Map name, character type (PMC/SCAV)
- UI elements for state machine transitions

### Key Requirements
- **Real-time inference**: <100ms latency for stream overlays
- **Cross-platform training/inference**: Train on AMD 7900 XTX, deploy on NVIDIA GPU
- **Clean gameplay capture**: No stream overlays/webcam interference
- **Standalone service**: Independent of Rust app, communicates via API

---

## Architecture Decision

### Standalone Python Microservice ⭐ RECOMMENDED

**Why Standalone:**
- Independent development and testing
- Easy to swap models without Rust recompile
- Can run on different machine if needed
- Simpler debugging and iteration

**System Architecture:**
```
Gaming PC (Tarkov)
    ↓
Elgato 4K X Capture Card
    ↓
    ├─→ Detection Service (Python)
    │   - Captures clean Elgato feed
    │   - YOLO + OCR inference (NVIDIA GPU)
    │   - WebSocket event broadcasting
    │   ↓
    │   WebSocket/REST API
    │   ↓
    └─→ OBS Studio
        - Adds webcam/overlays
        - Stream output

Rust App (tarkov_stream_producer)
    - Subscribes to detection events
    - Updates state machine
    - Database writes
    - OBS overlay updates
```

**Communication Protocol**: WebSocket (real-time event push)

---

## Hardware Setup

### Dev/Training System (Arch Linux)
- **CPU**: AMD Ryzen 9 9950X 16-Core (32 threads)
- **RAM**: 96GB DDR5
- **GPU**: AMD Radeon RX 7900 XTX (24GB VRAM)
- **ROCm**: 7.1.1 (latest, released Nov 2025)
- **Storage**: 3.6TB NVMe (main) + 3x 1TB drives
- **OS**: Arch Linux

**Training Optimizations:**
- Use `cache='ram'` in YOLO training (loads entire dataset to RAM)
- Batch size 64+ (utilize 96GB RAM)
- 32 worker threads for data loading (utilize 9950X cores)

### Gaming PC (Windows - Dual Boot)
- **Hardware**: Same as Dev/Training System (9950X, 96GB, 7900 XTX)
- **OS**: Windows (dual-boot with Arch Linux)
- **Purpose**: Game execution (Escape from Tarkov)
- **Restrictions**: Anticheat software limits what can run alongside game
- **Note**: Detection service will NOT run on this machine due to anticheat concerns

### Streaming/Inference System (Windows 11)
- **CPU**: AMD Ryzen 9 5900X (24 threads)
- **RAM**: 32GB DDR4
- **GPU**: NVIDIA GeForce RTX 5070 (16GB VRAM)
- **Capture**: Elgato 4K X (USB)
- **Storage**: NVMe SSD
- **OS**: Windows 11
- **Framework**: ONNX Runtime with TensorRT backend (NVIDIA)

**Inference Optimizations:**
- TensorRT acceleration on RTX 5070
- Capture directly from Elgato 4K X (before OBS overlays)
- Expected inference: 20-30ms per frame (30-50 FPS)

### Cross-Platform Strategy
1. Train models on AMD (PyTorch + ROCm)
2. Export to ONNX format (universal)
3. Deploy on NVIDIA (ONNX Runtime + TensorRT)
4. No code changes, just model files

---

## Video Capture Strategy

### Challenge: Stream Overlays
**Problem**: OBS Virtual Camera outputs the *composed* stream (webcam, alerts, BRB screens).
**Solution**: Capture clean Elgato feed *before* OBS adds overlays.

### Option 1: Direct Elgato Capture (Windows) ⭐ PREFERRED

**Concept**: Read from Elgato device directly using DirectShow (Windows).

```
Tarkov → Elgato 4K X → [Capture HERE] → OBS (adds overlays) → Stream
                            ↓
                    Detection Service
                    (clean gameplay)
```

**Implementation (Python + OpenCV):**
```python
import cv2

# Open Elgato as DirectShow device
cap = cv2.VideoCapture(DEVICE_ID, cv2.CAP_DSHOW)

# Configure for 1080p60
cap.set(cv2.CAP_PROP_FRAME_WIDTH, 1920)
cap.set(cv2.CAP_PROP_FRAME_HEIGHT, 1080)
cap.set(cv2.CAP_PROP_FPS, 60)
cap.set(cv2.CAP_PROP_FOURCC, cv2.VideoWriter_fourcc(*'MJPG'))

while True:
    ret, frame = cap.read()
    events = detect_tarkov_events(frame)
```

**Testing Required**: Check if Elgato allows shared access (OBS + Detection Service simultaneously).

**Test Script Location**: See [Testing & Validation](#testing--validation) section.

### Option 2: OBS NDI Output (Fallback)

If direct Elgato access fails (exclusive lock by OBS), use NDI.

**Setup:**
1. Install OBS NDI plugin
2. Configure NDI to output *Elgato source* (not composed scene)
3. Detection service reads NDI stream

**Pros:**
- Works around exclusive access issues
- Can output specific OBS source (clean feed)
- Low latency (~10-20ms)

**Cons:**
- Requires plugin installation
- Slight CPU overhead for NDI encoding

**Implementation (Python + NDI):**
```python
import NDIlib as ndi

# Find OBS Elgato NDI source
ndi.initialize()
sources = ndi.find_sources(timeout=5000)
obs_source = [s for s in sources if "Elgato" in s.name][0]

# Connect and receive
receiver = ndi.recv_create()
ndi.recv_connect(receiver, obs_source)

while True:
    frame = ndi.recv_capture(receiver, timeout=1000)
    # Convert NDI frame to numpy array
    img = np.frombuffer(frame.data, dtype=np.uint8)
    events = detect_tarkov_events(img)
```

### Video Preprocessing (for Training)

**Source**: OBS recordings or Elgato capture
**Format**: Mixed 4K and 1080p @ 60fps

**Preprocessing Steps:**
```bash
# Normalize to 1080p, reduce to 10fps (for labeling)
ffmpeg -i tarkov_gameplay.mp4 \
    -vf "scale=1920:1080" \
    -r 10 \
    -c:v libx264 -crf 23 \
    output_1080p_10fps.mp4
```

**Why 10fps?**
- 60fps has redundant consecutive frames
- 10fps = 600 frames/minute (sufficient for training)
- Reduces dataset size by 6x
- Speeds up labeling dramatically

---

## Framework & Models

### Deep Learning Framework: PyTorch + ROCm

**Why PyTorch:**
- Best ROCm support for AMD GPUs
- PyTorch 2.9 officially supported on ROCm 7.1.1
- Industry standard for computer vision
- Seamless ONNX export

**Installation (AMD Training System):**
```bash
# PyTorch with ROCm support
pip install torch torchvision --index-url https://download.pytorch.org/whl/rocm6.2
# Note: ROCm 7.1 wheels coming soon, use 6.2 for now

# Verify GPU detection
python -c "import torch; print(torch.cuda.is_available())"  # Should be True
python -c "import torch; print(torch.cuda.get_device_name(0))"  # Should show 7900 XTX
```

### Model Architecture: Hybrid Approach

**Two-Model System:**

#### 1. YOLOv8/YOLO11 for UI Element Detection
- **Purpose**: Detect kill notifications, death screens, extraction UI
- **Model**: Ultralytics YOLOv8n or YOLO11n (nano for speed)
- **Input**: 1920x1080 RGB frames
- **Output**: Bounding boxes + class labels
- **Classes**:
  - `kill_notification`
  - `death_screen`
  - `extraction_success`
  - `match_found`
  - `queue_timer`
  - `gameplay` (scene classifier)
  - `brb_screen` (scene classifier)

**Why YOLO:**
- 100+ FPS inference on modern GPUs
- Excellent for UI element detection
- Pre-trained models available (COCO dataset)
- Transfer learning accelerates training
- Proven for game UI detection

**References:**
- YOLOv8 UI detection: https://medium.com/@eslamelmishtawy/how-i-trained-yolov8-to-detect-mobile-ui-elements-using-the-vnis-dataset-f7f4b582fc09
- Ultralytics docs: https://docs.ultralytics.com/

#### 2. PaddleOCR for Text Recognition
- **Purpose**: Read map names, kill counts, character type, weapon names
- **Model**: PaddleOCRv3 with custom fine-tuning
- **Input**: Cropped text regions (from YOLO detections)
- **Output**: Text strings

**Why PaddleOCR:**
- Better for game text than EasyOCR
- Handles stylized fonts well
- Supports custom dataset training
- ~50ms per text region

**Fine-tuning**: Train on Tarkov-specific fonts/text styles.

**References:**
- PaddleOCR custom training: https://medium.com/@prishanga1/paddleocr-scene-text-recognition-in-the-wild-with-custom-dataset-59fd5f5cf6c3
- Training steps: https://gist.github.com/leonbora167/049ac6622b7a2fb5c23ec48070af486f

### Scene Classifier (Optional but Recommended)

**Purpose**: Detect when Tarkov is NOT active (BRB screens, desktop, menus).

**Implementation**: Simple CNN or YOLO classifier
- **Classes**: `gameplay`, `menu`, `brb_screen`, `desktop`
- **Action**: Pause detection when not in `gameplay` state

---

## Training Pipeline

### Data Collection

**Source**: OBS recordings of Tarkov gameplay
- Several hours of footage available (unlabeled)
- Mix of 4K and 1080p @ 60fps

**Preprocessing**:
```bash
#!/bin/bash
# preprocess_footage.sh

for video in raw_footage/*.mp4; do
    filename=$(basename "$video" .mp4)
    ffmpeg -i "$video" \
        -vf "scale=1920:1080" \
        -r 10 \
        -c:v libx264 -crf 23 \
        "processed_footage/${filename}_1080p_10fps.mp4"
done
```

### Labeling Tool: CVAT ⭐ RECOMMENDED

**CVAT (Computer Vision Annotation Tool)**
- Free, open-source video annotation tool
- Web-based UI (no installation for cloud version)
- Video interpolation (label frame 1 & 100, auto-labels 2-99)
- Export to YOLO format

**Why CVAT over Label Studio:**
- Optimized for video (interpolation, tracking)
- Faster for computer vision tasks
- Free with no trial period
- Best for object detection

**Setup Options:**

**Option 1: CVAT Cloud (Fastest)**
- Visit https://www.cvat.ai
- Sign up (free)
- Upload videos and start labeling

**Option 2: CVAT Local (Privacy)**
```bash
git clone https://github.com/cvat-ai/cvat
cd cvat
docker-compose up -d
# Access at http://localhost:8080
```

**Labeling Workflow:**
1. Upload 10fps preprocessed videos
2. Label key frames (every 10-30 frames)
3. Use interpolation for persistent elements
4. Label 200-300 frames total:
   - 50 frames: Queue screens
   - 100 frames: Kill notifications (various types)
   - 50 frames: Death screens
   - 50 frames: Extraction screens
   - 20 frames: BRB/menu screens (scene classifier)
5. Export to YOLO format

**References:**
- CVAT vs Label Studio: https://www.cvat.ai/resources/blog/cvat-or-label-studio-which-one-to-choose
- CVAT GitHub: https://github.com/cvat-ai/cvat

### Training: YOLOv8 (AMD 7900 XTX)

**Setup:**
```bash
pip install ultralytics
```

**Training Script:**
```python
from ultralytics import YOLO

# Load pre-trained model (transfer learning)
model = YOLO('yolov8n.pt')  # or yolo11n.pt

# Train with optimizations for 96GB RAM
model.train(
    data='tarkov_ui.yaml',        # Dataset config
    epochs=100,
    imgsz=1920,                   # Full HD training
    batch=64,                     # Large batch (96GB RAM!)
    device='cuda',                # ROCm appears as CUDA
    workers=32,                   # Utilize 9950X cores
    cache='ram',                  # Cache dataset in RAM (FAST!)
    project='tarkov_detector',
    name='yolov8n_tarkov'
)

# Export to ONNX for cross-platform inference
model.export(format='onnx', opset=17)
```

**Dataset Config (`tarkov_ui.yaml`):**
```yaml
path: /path/to/dataset
train: images/train
val: images/val

nc: 7  # Number of classes
names:
  0: kill_notification
  1: death_screen
  2: extraction_success
  3: match_found
  4: queue_timer
  5: gameplay
  6: brb_screen
```

**Expected Training Time**: 3-5 hours (200-300 images, 100 epochs)

**Performance Expectations (7900 XTX):**
- VRAM usage: ~16GB (plenty of headroom with 24GB)
- Bottleneck: Data loading (optimized with RAM cache + NVMe)
- Benchmark reference: https://cprimozic.net/notes/posts/machine-learning-benchmarks-on-the-7900-xtx/

### Training: PaddleOCR Fine-tuning (Optional)

**Setup:**
```bash
pip install paddlepaddle-gpu paddleocr
```

**Fine-tuning on Tarkov Text:**
- Use OCR-Dataset-Generator: https://github.com/xReniar/OCR-Dataset-Generator
- Generate synthetic Tarkov text images (weapon names, kill messages)
- Fine-tune PaddleOCR recognition model
- Expected time: 1-2 hours

**Reference**: https://medium.com/@prishanga1/paddleocr-scene-text-recognition-in-the-wild-with-custom-dataset-59fd5f5cf6c3

---

## Detection Service Design

### Tech Stack

- **Web Framework**: FastAPI (async, fast, modern)
- **Inference**: ONNX Runtime (cross-platform, TensorRT backend)
- **Communication**: WebSocket (real-time event push)
- **Video Capture**: OpenCV (DirectShow on Windows)

### Service Architecture

```
┌─────────────────────────────────────────┐
│  Detection Service (Python)             │
│                                         │
│  ┌───────────────────────────────────┐ │
│  │  Capture Thread                   │ │
│  │  - Read from Elgato/NDI           │ │
│  │  - 60fps frame capture            │ │
│  └─────────────┬─────────────────────┘ │
│                ↓                        │
│  ┌───────────────────────────────────┐ │
│  │  Detection Thread                 │ │
│  │  - YOLO inference (every 6th frame)│ │
│  │  - OCR on detected text regions   │ │
│  │  - Scene classification           │ │
│  └─────────────┬─────────────────────┘ │
│                ↓                        │
│  ┌───────────────────────────────────┐ │
│  │  Event Queue                      │ │
│  │  - Buffer detected events         │ │
│  │  - Temporal filtering             │ │
│  └─────────────┬─────────────────────┘ │
│                ↓                        │
│  ┌───────────────────────────────────┐ │
│  │  WebSocket Server                 │ │
│  │  - Broadcast events to clients    │ │
│  │  - Health monitoring              │ │
│  └───────────────────────────────────┘ │
└─────────────────────────────────────────┘
```

### API Design

#### WebSocket Endpoint: `/ws`

**Client (Rust app) connects and receives events:**

```json
{
  "type": "kill",
  "timestamp": 1702564800.123,
  "confidence": 0.95,
  "data": {
    "enemy_type": "scav",
    "weapon": "AK-74M",
    "headshot": true
  }
}
```

**Event Types:**
- `raid_start`: Queue entered
- `match_found`: Loading screen detected
- `kill`: Kill notification detected
- `death`: Player died
- `extraction`: Raid ended (survived)
- `scene_change`: Switched to BRB/menu (pause detection)

#### REST Endpoints

- `GET /health`: Service health check
  ```json
  {
    "status": "running",
    "fps": 58.3,
    "latency_ms": 45,
    "clients_connected": 1,
    "gpu_usage": 45.2
  }
  ```

- `GET /stats`: Detection statistics
- `POST /config`: Update detection settings (confidence thresholds, etc.)

### Implementation Skeleton

**File Structure:**
```
tarkov_detection_service/
├── detection_service.py       # Main FastAPI app
├── capture.py                 # Elgato/NDI capture
├── detector.py                # YOLO + OCR inference
├── models/
│   ├── yolov8n_tarkov.onnx
│   └── paddleocr_weights/
├── config.yaml                # Configuration
└── requirements.txt
```

**Main Service (`detection_service.py`):**
```python
from fastapi import FastAPI, WebSocket
import asyncio
import cv2
from detector import TarkovDetector
from capture import ElgatoCapture

app = FastAPI()
detector = TarkovDetector('models/yolov8n_tarkov.onnx')
capture = ElgatoCapture(device_id=1)

connected_clients = []

@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    await websocket.accept()
    connected_clients.append(websocket)
    try:
        while True:
            await asyncio.sleep(0.1)
    except:
        connected_clients.remove(websocket)

async def broadcast_event(event):
    for client in connected_clients:
        try:
            await client.send_json(event)
        except:
            pass

async def detection_loop():
    frame_count = 0
    while True:
        frame = capture.read_frame()

        # Detect every 6th frame (60fps -> 10 detections/sec)
        if frame_count % 6 == 0:
            events = detector.detect(frame)
            for event in events:
                await broadcast_event(event)

        frame_count += 1
        await asyncio.sleep(0.001)

@app.on_event("startup")
async def startup():
    asyncio.create_task(detection_loop())

@app.get("/health")
async def health():
    return {"status": "running", "clients": len(connected_clients)}
```

---

## Testing & Validation

### Test 1: Elgato Direct Access (Windows)

**Purpose**: Verify Elgato device can be accessed while OBS is running.

**Test Script (`test_elgato_access.py`):**
```python
import cv2

def find_elgato_device():
    """Find Elgato device ID"""
    print("Searching for video devices...")

    for i in range(10):
        cap = cv2.VideoCapture(i, cv2.CAP_DSHOW)
        if cap.isOpened():
            width = cap.get(cv2.CAP_PROP_FRAME_WIDTH)
            height = cap.get(cv2.CAP_PROP_FRAME_HEIGHT)
            print(f"Device {i}: {width}x{height}")

            ret, frame = cap.read()
            if ret:
                print(f"  ✅ Can read frames (shape: {frame.shape})")
            else:
                print(f"  ❌ Device opened but can't read frames")

            cap.release()

def test_capture_with_obs():
    """Test if we can capture while OBS is running"""
    device_id = int(input("\nWhich device is your Elgato? Enter ID: "))

    cap = cv2.VideoCapture(device_id, cv2.CAP_DSHOW)

    if not cap.isOpened():
        print("❌ FAILED: Can't open Elgato (exclusive access)")
        return False

    # Try reading 30 frames
    success_count = 0
    for i in range(30):
        ret, frame = cap.read()
        if ret:
            success_count += 1

    cap.release()

    if success_count == 30:
        print(f"✅ SUCCESS: Can read Elgato while OBS is running!")
        return True
    else:
        print(f"⚠️ PARTIAL: Read {success_count}/30 frames")
        return False

if __name__ == "__main__":
    print("1. First, CLOSE OBS completely")
    input("Press Enter when OBS is closed...")
    find_elgato_device()

    print("\n2. Now, OPEN OBS and start using the Elgato")
    input("Press Enter when OBS is running...")

    if test_capture_with_obs():
        print("\n✅ Direct Elgato access works! Use Option 1.")
    else:
        print("\n❌ Direct access blocked. Use Option 2 (NDI).")
```

**Run this test FIRST** to determine capture strategy.

### Test 2: YOLO Inference Performance

**Purpose**: Validate model performance on target hardware.

```python
from ultralytics import YOLO
import cv2
import time
import numpy as np

model = YOLO('models/yolov8n_tarkov.onnx')

# Create dummy 1080p frame
frame = np.random.randint(0, 255, (1080, 1920, 3), dtype=np.uint8)

# Warmup
for _ in range(10):
    model.predict(frame, conf=0.5)

# Benchmark
times = []
for _ in range(100):
    start = time.perf_counter()
    results = model.predict(frame, conf=0.5)
    times.append((time.perf_counter() - start) * 1000)

print(f"Average inference time: {np.mean(times):.1f}ms")
print(f"FPS: {1000 / np.mean(times):.1f}")
print(f"P95 latency: {np.percentile(times, 95):.1f}ms")
```

**Target**: <50ms per frame on NVIDIA GPU (20+ FPS)

### Test 3: End-to-End Latency

**Purpose**: Measure capture → detection → broadcast latency.

```python
import time

# Measure full pipeline
start = time.perf_counter()

frame = capture.read_frame()              # ~10ms
events = detector.detect(frame)           # ~50ms
await broadcast_event(events[0])          # ~5ms

total_latency = (time.perf_counter() - start) * 1000
print(f"End-to-end latency: {total_latency:.1f}ms")
```

**Target**: <100ms total latency

---

## Integration with Rust App

### Rust WebSocket Client

**Dependencies (`Cargo.toml`):**
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.20"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

**WebSocket Client (`src/detection_client.rs`):**
```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct DetectionEvent {
    r#type: String,
    timestamp: f64,
    confidence: f32,
    data: serde_json::Value,
}

pub async fn listen_for_events() -> Result<(), Box<dyn std::error::Error>> {
    let url = "ws://localhost:8000/ws";
    let (ws_stream, _) = connect_async(url).await?;

    let (_, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        let msg = msg?;

        if let Message::Text(text) = msg {
            let event: DetectionEvent = serde_json::from_str(&text)?;
            handle_detection_event(event).await?;
        }
    }

    Ok(())
}

async fn handle_detection_event(event: DetectionEvent) -> Result<(), Box<dyn std::error::Error>> {
    match event.r#type.as_str() {
        "kill" => {
            // Update state machine: add kill to current raid
            tracing::info!("Kill detected: {:?}", event.data);
            // TODO: Call your state machine update function
        }
        "death" => {
            // Update state machine: end raid (died)
            tracing::info!("Death detected");
            // TODO: Call raid end function
        }
        "extraction" => {
            // Update state machine: end raid (survived)
            tracing::info!("Extraction detected");
            // TODO: Call raid end function
        }
        _ => {
            tracing::debug!("Unknown event type: {}", event.r#type);
        }
    }

    Ok(())
}
```

**Main App Integration (`src/main.rs`):**
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Spawn detection event listener
    tokio::spawn(async {
        if let Err(e) = detection_client::listen_for_events().await {
            tracing::error!("Detection client error: {}", e);
        }
    });

    // Continue with rest of app (database, API server, etc.)
    // ...

    Ok(())
}
```

### State Machine Integration

**Event Flow:**
```
Detection Service detects kill
    ↓
WebSocket event sent
    ↓
Rust app receives event
    ↓
State machine transition: in_progress + increment kill_count
    ↓
Database update (INSERT INTO kills)
    ↓
OBS overlay update (update kill counter text file)
```

**Confidence Thresholds:**
- Only trigger state transitions for high-confidence events (>0.8)
- Log lower-confidence events for review
- Allow manual override via Stream Deck (Phase 2b still available)

---

## Implementation Timeline

### Phase 4a: Data Preparation (Week 1)
**Estimated**: 4-6 hours

1. **Preprocess footage** (1h)
   - Normalize to 1080p @ 10fps
   - Organize into folders

2. **CVAT setup** (30min)
   - Install locally or sign up for cloud
   - Upload sample videos

3. **Labeling** (2-3h)
   - Label 200-300 frames
   - Use interpolation to accelerate
   - Export to YOLO format

4. **Test Elgato capture** (30min)
   - Run test script
   - Determine capture strategy (direct vs NDI)

### Phase 4b: Model Training (Week 2)
**Estimated**: 6-8 hours (mostly compute time)

1. **Setup PyTorch + ROCm** (1h)
   - Install dependencies
   - Verify GPU detection

2. **Train YOLOv8** (4-5h compute, 1h active)
   - Configure training script
   - Monitor training progress
   - Validate on test set

3. **Export to ONNX** (30min)
   - Export model
   - Test inference on NVIDIA system

4. **Fine-tune PaddleOCR** (1-2h) - OPTIONAL
   - Generate synthetic Tarkov text
   - Fine-tune recognition model

### Phase 4c: Detection Service (Week 3)
**Estimated**: 6-8 hours

1. **Implement capture module** (2h)
   - Elgato DirectShow or NDI
   - Frame buffering

2. **Implement detection module** (2h)
   - ONNX inference
   - Scene classification
   - Event filtering

3. **FastAPI service** (2h)
   - WebSocket server
   - Health endpoints
   - Configuration

4. **Testing** (2h)
   - End-to-end latency
   - Accuracy validation
   - Stress testing

### Phase 4d: Rust Integration (Week 4)
**Estimated**: 4-6 hours

1. **WebSocket client** (2h)
   - Connect to detection service
   - Event deserialization

2. **State machine integration** (2h)
   - Map events to state transitions
   - Database updates

3. **Testing** (2h)
   - Full pipeline test with live gameplay
   - Verify state machine correctness
   - Measure accuracy

### Total Estimated Time: 20-28 hours

---

## References

### Research & Documentation

**ROCm & PyTorch:**
- ROCm 7.1.1 Release Notes: https://www.amd.com/en/resources/support-articles/release-notes/RN-AMDGPU-UNIFIED-LINUX-25-30-1-ROCM-7-1-1.html
- AMD ROCm + PyTorch 7900 XTX Support: https://www.phoronix.com/news/RX-7900-XTX-ROCm-PyTorch
- 7900 XTX ML Benchmarks: https://cprimozic.net/notes/posts/machine-learning-benchmarks-on-the-7900-xtx/

**YOLO Models:**
- YOLOv8 UI Element Detection: https://medium.com/@eslamelmishtawy/how-i-trained-yolov8-to-detect-mobile-ui-elements-using-the-vnis-dataset-f7f4b582fc09
- Ultralytics YOLO Documentation: https://docs.ultralytics.com/modes/predict/
- YOLO Game Object Detection: https://betterprogramming.pub/how-to-train-yolov5-for-recognizing-custom-game-objects-in-real-time-9d78369928a8

**Annotation Tools:**
- CVAT vs Label Studio: https://www.cvat.ai/resources/blog/cvat-or-label-studio-which-one-to-choose
- CVAT GitHub: https://github.com/cvat-ai/cvat

**OCR:**
- PaddleOCR Custom Training: https://medium.com/@prishanga1/paddleocr-scene-text-recognition-in-the-wild-with-custom-dataset-59fd5f5cf6c3
- PaddleOCR Training Steps: https://gist.github.com/leonbora167/049ac6622b7a2fb5c23ec48070af486f
- OCR Dataset Generator: https://github.com/xReniar/OCR-Dataset-Generator

**Video Event Detection:**
- Sports Video Event Detection: https://arxiv.org/html/2505.03991v3
- Esports Scene Detection: https://www.nature.com/articles/s41598-025-93692-0

---

## Notes & Considerations

### Performance Optimization
- **Frame skipping**: Detect every 6th frame (60fps → 10 detections/sec) reduces GPU load
- **Batch inference**: Process multiple frames in batches for efficiency
- **Model quantization**: Consider INT8 quantization for faster inference (trade-off: accuracy)

### Accuracy Improvements
- **Temporal filtering**: Require 2-3 consecutive detections before triggering event (reduce false positives)
- **Confidence calibration**: Tune thresholds per event type
- **Active learning**: Periodically review low-confidence detections and add to training set

### Failure Modes
- **Detection service crashes**: Rust app should fall back to manual mode (Stream Deck)
- **Model drift**: Game UI updates may require retraining
- **Network issues**: WebSocket reconnection logic needed

### Future Enhancements
- **Multi-game support**: Train separate models for other games
- **Cloud deployment**: Run detection service on cloud GPU for lower local overhead
- **Model versioning**: A/B test new models before full deployment

---

**Document Status**: Ready for implementation when Phase 2 & 3 are complete.
