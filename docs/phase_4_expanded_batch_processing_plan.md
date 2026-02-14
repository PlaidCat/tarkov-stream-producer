# Phase 4 Expanded: AI Vision + Batch Processing & Backfill Pipeline

## Context

The Tarkov Stream Producer project needs an AI vision system that can:
1. **Watch live gameplay** via Elgato capture and detect game events in real-time
2. **Process pre-recorded sessions** (50-100+ hours of MKV recordings from OBS) through the same detection pipeline
3. **Backfill the database** with historical data by importing detected events via the REST API

The existing Phase 4 plan (`docs/phase_4_video_detection_plan.md`) covers live detection well but lacks batch processing and backfilling. This plan expands it with parallel development tracks: training pipeline + batch processing tool, both feeding into the same YOLO + PaddleOCR detection models.

**Key constraint**: All database writes go through the Rust REST API (no direct Python → SQLite). Phase 2b must be completed far enough to support the import workflow.

---

## Architecture Overview

```
                    ┌──────────────────────────────┐
                    │   Video Source Abstraction    │
                    │                              │
                    │  ┌────────┐   ┌───────────┐ │
                    │  │  Live  │   │   Batch   │ │
                    │  │Elgato/ │   │ MKV files │ │
                    │  │  NDI   │   │ (OpenCV)  │ │
                    │  └───┬────┘   └─────┬─────┘ │
                    │      └──────┬───────┘       │
                    └─────────────┼───────────────┘
                                  ↓
                    ┌──────────────────────────────┐
                    │   Detection Pipeline         │
                    │   (shared, identical)         │
                    │                              │
                    │   YOLO11n → UI detection     │
                    │   PaddleOCR → text extract   │
                    │   Temporal filter → events   │
                    └─────────────┬───────────────┘
                                  ↓
              ┌───────────────────┼───────────────────┐
              ↓                                       ↓
    ┌──────────────────┐               ┌──────────────────────┐
    │   Live Mode      │               │   Batch Mode         │
    │   WebSocket →    │               │   JSON event log →   │
    │   Rust app       │               │   Review UI →        │
    │   (real-time)    │               │   REST API import    │
    └──────────────────┘               └──────────────────────┘
              │                                       │
              └───────────────────┬───────────────────┘
                                  ↓
                    ┌──────────────────────────────┐
                    │   Rust App (REST API)        │
                    │   → SQLite Database          │
                    │   → OBS Overlays             │
                    └──────────────────────────────┘
```

---

## What Needs to Be Detected

Based on the 4-table schema (`stream_sessions`, `raids`, `raid_state_transitions`, `kills`):

### High Priority (Core functionality)
| Event | Detection Method | Database Action |
|-------|-----------------|-----------------|
| Kill notification | YOLO bbox → OCR text | `add_kill(enemy_type, weapon, headshot)` |
| Survived screen | YOLO + OCR "SURVIVED" | `end_raid()` + terminal state |
| Killed in Action | YOLO + OCR "KILLED IN ACTION" | `end_raid()` + terminal state |
| Missing in Action | YOLO + OCR "MISSING IN ACTION" | `end_raid()` + terminal state |
| Queue/matchmaking | YOLO queue timer | `log_state_transition("queuing")` |
| In-raid gameplay | Scene classifier (FPS HUD) | `log_state_transition("raid_active")` |

### Medium Priority (Rich analytics)
| Event | Detection Method | Database Action |
|-------|-----------------|-----------------|
| Map name | OCR on selection screen | `create_raid(map_name)` |
| PMC vs Scav | UI button detection | `create_raid(character_type)` |
| PVE vs PVP | Checkbox detection (pre-raid only) | `create_raid(game_mode)` |
| Extract location | OCR on survived screen | `end_raid(extract_location)` |
| Loading states | Loading screen detection | State transitions |
| Post-raid review | Kill list/stats screen | `log_state_transition("post_raid_review")` |

### Session-level data (manual input for batch)
- `session_type` (stream/practice/casual) — user provides per video
- `started_at` — extracted from MKV metadata or user-provided
- `notes` — optional, user-provided

---

## REST API Endpoints Needed for Backfill

**Already built (Phase 2b.2):**
- `POST /api/session` — create session (with started_at override needed)
- `GET /api/session/current` — get active session
- `POST /api/session/end` — end session

**Still needed (Phase 2b.3+):**
- `POST /api/raid` — create raid with map, character, mode
- `GET /api/raid/current` — get active raid
- `POST /api/raid/{id}/transition` — log state transition
- `POST /api/raid/{id}/kill` — add kill event
- `POST /api/raid/{id}/end` — end raid with outcome

**Backfill-specific additions needed:**
- Timestamp override support on all POST endpoints (accept optional `timestamp` field in request body so batch imports can specify historical times instead of `now()`)
- `POST /api/session` needs optional `started_at` in request body
- Kill/transition endpoints need optional timestamp fields

This is critical: without timestamp overrides, all backfilled data would have `now()` timestamps instead of when the events actually happened.

---

## Implementation Plan — Parallel Tracks

### Track A: Training Pipeline (Dev System, Arch Linux)
### Track B: Batch Processing + Backfill Tools (Dev System)
### Track C: REST API Extensions (Phase 2b completion)

These run in parallel. Track C unblocks the final import step of Track B.

---

### Phase 4.1: Data Preparation & Labeling (Track A — Week 1-2)
**Estimated: 8-12 hours active work**
**Hardware: Dev System (Arch Linux)**

1. **Preprocess MKV footage** (2h)
   - Normalize to 1080p @ 10fps for labeling (reduces volume ~6x)
   - MKV → MP4 conversion not strictly needed (OpenCV reads MKV), but 10fps extraction reduces labeling effort
   - `ffmpeg -i input.mkv -vf "scale=1920:1080" -r 10 output_10fps.mp4`
   - Select diverse clips: different maps, PMC/Scav, PVE/PVP, kills, deaths, extractions

2. **CVAT setup** (1h)
   - Run CVAT locally via Docker on Dev System (privacy, no upload limits)
   - `docker-compose up -d` from CVAT repo
   - Create project with label classes: `kill_notification`, `death_screen`, `survived_screen`, `mia_screen`, `queue_timer`, `match_found`, `gameplay_hud`, `loading_screen`, `post_raid_screen`

3. **Frame labeling** (5-8h)
   - Target: 300-500 labeled frames (higher than original 200-300 due to 50-100h video library)
   - Prioritize: 150 kill notifications (variety of enemy types), 80 terminal screens, 80 queue/loading, 50 gameplay HUD, 40 post-raid
   - Use CVAT video interpolation to speed up persistent UI elements
   - Export to YOLO format (txt files with bounding boxes)

4. **OCR training data** (optional, 1-2h)
   - Crop text regions from labeled frames
   - Create ground truth text file for PaddleOCR fine-tuning
   - Focus on weapon names and enemy types (most variable text)

**Deliverables:**
- Labeled dataset in YOLO format (images/ + labels/ directories)
- Dataset YAML config for training
- CVAT project backup for future labeling iterations

---

### Phase 4.2: Model Training (Track A — Week 2-3)
**Estimated: 6-8 hours (mostly compute time, ~2h active)**
**Hardware: Dev System (AMD 7900 XTX + ROCm)**

1. **Environment setup** (1h)
   - Install PyTorch + ROCm (`pip install torch torchvision --index-url https://download.pytorch.org/whl/rocm6.2`)
   - Install Ultralytics (`pip install ultralytics`)
   - Verify: `python -c "import torch; print(torch.cuda.get_device_name(0))"` → "AMD Radeon RX 7900 XTX"

2. **Train YOLO11n** (4-5h compute, 30min active)
   - Transfer learning from pre-trained COCO weights
   - Config: `imgsz=1920, batch=64, epochs=100, cache='ram', workers=32`
   - Monitor: mAP@50, loss curves, per-class accuracy
   - Validate on held-out test set (20% split)

3. **Export to ONNX** (30min)
   - `model.export(format='onnx', opset=17)`
   - Test ONNX inference on same validation set
   - Verify: exported model produces same predictions as PyTorch model

4. **PaddleOCR evaluation** (1h)
   - Test stock PaddleOCR on cropped Tarkov text regions
   - If accuracy >90%, skip fine-tuning; otherwise fine-tune on Tarkov font samples
   - Expected: stock PaddleOCR handles most Tarkov text well (clean fonts on dark backgrounds)

**Deliverables:**
- `yolo11n_tarkov_v1.onnx` — trained YOLO model
- Training metrics (mAP, confusion matrix)
- PaddleOCR evaluation report (fine-tune or not)

---

### Phase 4.3: Video Input Abstraction + Detection Service Core (Track B — Week 2-3)
**Estimated: 8-10 hours**
**Hardware: Dev System**

1. **Video source abstraction** (`video_input.py`, 3h)
   - `VideoSource` ABC with `read_frame()`, `seek()`, `get_fps()`, `close()`
   - `Frame` dataclass: image, frame_number, video_timestamp (seconds from start), real_timestamp (datetime)
   - `LiveCaptureSource` — Elgato/NDI DirectShow capture
   - `BatchFileSource` — MKV/MP4 file processing via OpenCV
     - Constructor takes: `video_path`, `session_start_time` (datetime), `detection_fps` (default 10)
     - Timestamp mapping: `real_timestamp = session_start_time + timedelta(seconds=frame_number/fps)`
   - `SmartBatchFileSource` — auto-detects start time from MKV metadata via `ffprobe`
     - OBS embeds creation_time in MKV container metadata
     - Fallback: file modification time

2. **Detection pipeline** (`detector.py`, 3h)
   - `TarkovDetector` class wrapping YOLO + PaddleOCR
   - Accepts `Frame` objects (not raw OpenCV frames)
   - Returns `List[DetectedEvent]` with: event_type, confidence, data dict, frame reference
   - Temporal filtering: require N consecutive detections before emitting event (configurable, default 3)
     - In batch mode: consecutive = within N frames at detection_fps
     - In live mode: consecutive = within N × frame_interval seconds
   - OCR pipeline: YOLO detects bbox → crop region → PaddleOCR extracts text → regex parse

3. **State machine** (`state_machine.py`, 2h)
   - Tracks current game state (idle → stash → pre_raid → queue → deploying → raid_active → terminal)
   - Validates transitions (can't go from idle → survived)
   - Emits higher-level events: raid_started, raid_ended, kill_detected, state_changed
   - Same logic for both live and batch modes

4. **Event output** (2h)
   - JSON event log writer for batch mode
   - WebSocket broadcaster for live mode
   - Unified `EventEmitter` interface

**Key file structure:**
```
tarkov_detection_service/
├── video_input.py          # VideoSource abstraction
├── detector.py             # YOLO + PaddleOCR inference
├── state_machine.py        # Game state tracking
├── event_emitter.py        # Output: JSON log or WebSocket
├── batch_process.py        # CLI for batch processing
├── import_events.py        # CLI for REST API import
├── review_server.py        # Flask review UI
├── detection_service.py    # FastAPI live service
├── models/
│   ├── yolo11n_tarkov.onnx
│   └── paddleocr_weights/
├── config.yaml
├── requirements.txt
└── templates/
    └── review.html
```

---

### Phase 4.4: Batch Processing CLI (Track B — Week 3-4)
**Estimated: 6-8 hours**

1. **`batch_process.py` script** (3h)
   - CLI args: `--video`, `--start-time` (optional, auto-detect from MKV), `--session-type`, `--output`, `--detection-fps`, `--confidence-threshold`
   - Progress bar (tqdm) showing frames processed / total
   - Saves thumbnails for each detected event (for review)
   - Outputs JSON event log (see format below)

2. **Multi-video batch processing** (2h)
   - `--video-dir` mode: process all MKV files in a directory
   - Sort by file creation time, process sequentially
   - Separate JSON output per video
   - Summary report: total events detected across all videos

3. **Performance optimization** (2h)
   - Target: process 50-100h of footage efficiently on Dev System
   - At 10fps detection rate: 1 hour of video = 36,000 frames to detect on
   - YOLO11n inference ~5-10ms/frame on 7900 XTX → ~600 frames/sec → 1 hour of video in ~60 seconds
   - Bottleneck will be video decoding, not inference
   - Add frame skipping: detect every Nth frame based on `detection_fps`

**JSON Event Log Format:**
```json
{
  "metadata": {
    "video_path": "/recordings/2026-01-15_stream.mkv",
    "session_start_time": "2026-01-15T18:00:00Z",
    "session_type": "stream",
    "video_duration_seconds": 10800,
    "total_frames": 648000,
    "detection_fps": 10,
    "model_version": "yolo11n_tarkov_v1",
    "processed_at": "2026-02-15T14:30:00Z"
  },
  "events": [
    {
      "event_id": "evt_001",
      "type": "state_transition",
      "timestamp": "2026-01-15T18:05:23Z",
      "video_offset_seconds": 323.0,
      "frame_number": 19380,
      "confidence": 0.95,
      "data": { "from_state": "idle", "to_state": "pre_raid_setup" },
      "thumbnail": "thumbnails/evt_001.jpg"
    },
    {
      "event_id": "evt_002",
      "type": "raid_start",
      "timestamp": "2026-01-15T18:06:10Z",
      "video_offset_seconds": 370.0,
      "frame_number": 22200,
      "confidence": 0.92,
      "data": { "map_name": "customs", "character_type": "pmc", "game_mode": "pve" },
      "thumbnail": "thumbnails/evt_002.jpg"
    }
  ]
}
```

---

### Phase 4.5: Review & Correction UI (Track B — Week 4)
**Estimated: 6-8 hours**

1. **Flask review server** (`review_server.py`, 3h)
   - Serves HTML page with event timeline
   - Routes: GET `/` (timeline view), POST `/api/update_event`, POST `/api/delete_event`, POST `/api/add_event`
   - Events with confidence < 0.8 highlighted for review
   - Video seek: click event → jump to that timestamp in embedded video player

2. **Review HTML template** (`templates/review.html`, 3h)
   - Chronological event list with thumbnails
   - Inline editing of event data (enemy_type, weapon, map_name)
   - Add missing events manually (click "add event at timestamp")
   - Delete false positives
   - Keyboard shortcuts for efficient review (j/k navigate, e edit, d delete, a approve)

3. **Export corrected events** (1h)
   - Save corrected JSON (same format, with `"manually_reviewed": true` flag)
   - Export reviewed frames as CVAT annotations for retraining (closes feedback loop)

**Review workflow for 50-100h of footage:**
- Batch process generates ~500-2000 events per hour of footage (varies by gameplay)
- Low-confidence events (~10-20%) flagged for review
- Active learning selector prioritizes most valuable frames to review
- Estimated review time: 5-10 minutes per hour of processed footage

---

### Phase 4.6: REST API Extensions for Backfill (Track C — Week 3-4)
**Estimated: 8-12 hours (continues Phase 2b)**

This is the **critical dependency** for importing batch-processed data. The existing Phase 2b plan covers raid endpoints, but backfilling needs timestamp override support.

1. **Complete Phase 2b.3: Raid Endpoints** (6h, TDD)
   - `POST /api/raid` — create raid (map, character_type, game_mode)
   - `GET /api/raid/current` — get active raid
   - `POST /api/raid/{id}/transition` — log state transition
   - `POST /api/raid/{id}/kill` — add kill
   - `POST /api/raid/{id}/end` — end raid (outcome, extract_location)

2. **Add timestamp override fields** (3h)
   - All create/mutate endpoints accept optional `timestamp` field in JSON body
   - When present, use that timestamp instead of `now()`
   - Already supported by DB layer (`Option<OffsetDateTime>` parameters)
   - DTOs need: `pub timestamp: Option<String>` (ISO 8601), parsed in handler
   - Example: `POST /api/raid/1/kill` with `{"enemy_type": "scav", "weapon": "AK-74M", "timestamp": "2026-01-15T18:12:34Z"}`

3. **Batch import endpoint** (optional, 2h)
   - `POST /api/import/session` — accepts full session event log, creates everything in a transaction
   - Alternative to calling individual endpoints sequentially
   - Advantage: atomic (all or nothing), faster (one HTTP call)

**Files to modify:**
- `src/api/dto.rs` — add timestamp fields to request DTOs
- `src/api/handlers/session.rs` — pass timestamp to DB functions
- `src/api/handlers/raid.rs` (new) — raid CRUD handlers
- `src/api/routes.rs` — mount new routes

---

### Phase 4.7: Import Tool (Track B — Week 5)
**Estimated: 6-8 hours**

1. **`import_events.py` script** (4h)
   - Reads corrected JSON event log
   - Calls Rust REST API endpoints in sequence:
     1. `POST /api/session` (with start timestamp)
     2. For each detected raid: create raid → transitions → kills → end raid
     3. `POST /api/session/end` (with end timestamp)
   - Dry-run mode: print all API calls without executing
   - Handles errors: retry on transient failures, report permanent failures
   - Duplicate detection: check if session with same start time already exists

2. **Multi-video import** (2h)
   - Process directory of corrected JSON files
   - Import in chronological order
   - Summary report: sessions created, raids, kills totals

3. **Validation & testing** (2h)
   - End-to-end test: MKV → batch process → review → import → verify DB
   - Query stats after import to verify data integrity
   - Compare manual event count against detected count

---

### Phase 4.8: Training Feedback Loop (Ongoing)
**Estimated: 4-6 hours setup, then ongoing**

1. **Export corrections to CVAT** (2h)
   - Reviewed/corrected events → CVAT XML annotations
   - Import into CVAT project alongside original labels
   - Augment training dataset

2. **Active learning selector** (2h)
   - Score events by uncertainty (confidence 0.5-0.7 = most informative)
   - Prioritize underrepresented event types
   - Select top N frames per batch for focused review

3. **Retraining cycle** (2h per iteration)
   - Iteration 1: 300-500 manual labels → train v1 (baseline ~60-70%)
   - Iteration 2: Process videos with v1 → review corrections → retrain v2 (~75-80%)
   - Iteration 3: Active learning selection → focused review → retrain v3 (~85-90%)
   - Each iteration: ~2h active work (labeling corrections + training launch)

---

## Hardware Allocation

| Task | System | Why |
|------|--------|-----|
| CVAT labeling | Dev System | Docker, big screen, no time pressure |
| Model training | Dev System | 7900 XTX (24GB VRAM) + ROCm + 96GB RAM |
| Batch processing (50-100h) | Dev System | Fastest GPU, process overnight |
| Review UI | Dev System | Just a Flask server + browser |
| Live detection (streams) | Streaming PC | RTX 5070 + TensorRT, receives Elgato feed |
| REST API / DB | Either | Rust app runs on whichever system |

---

## MKV-Specific Considerations

- OpenCV's `VideoCapture` reads MKV files natively (no conversion needed)
- OBS stores creation timestamp in MKV container metadata → extract via `ffprobe -show_format`
- MKV supports variable frame rate — need to handle via `CAP_PROP_POS_MSEC` instead of frame counting
- For 10fps detection sampling: use video timestamp (milliseconds) not frame index
- No remux to MP4 needed unless specific tools require it

---

## Timeline Summary

| Week | Track A (Training) | Track B (Batch Pipeline) | Track C (REST API) |
|------|-------------------|--------------------------|-------------------|
| 1 | Preprocess footage, CVAT setup, begin labeling | — | — |
| 2 | Finish labeling, begin training | Video input abstraction, detector core | — |
| 3 | Export ONNX model, PaddleOCR eval | Batch CLI, state machine | Raid endpoints (TDD) |
| 4 | — | Review UI | Timestamp override support |
| 5 | — | Import tool, integration testing | — |
| 6 | Retraining iteration | End-to-end validation | — |

**Total: ~50-65 hours across 6 weeks**

---

## Verification Plan

1. **Model accuracy**: Process 1 hour of test video, manually count events, compare to detection count. Target: >85% recall, >90% precision
2. **Timestamp accuracy**: Compare detected event timestamps against known events (e.g., manually note a kill time, verify within ±2 seconds)
3. **Import integrity**: After importing a session, run existing stats functions (`calculate_time_between_raids`, K/D ratio, etc.) and verify results make sense
4. **Round-trip consistency**: Process same video twice → same events detected (deterministic)
5. **Live vs batch consistency**: Play back a recording through both live simulation and batch mode → same events detected
