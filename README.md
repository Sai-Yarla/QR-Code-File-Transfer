# QR Code File Transfer

A high-speed, air-gapped file transfer system using dynamically generated QR codes. The project allows you to transfer files from a computer to an Android smartphone using only a sequence of QR codes displayed on the screen and scanned by the phone's camera, without relying on Wi-Fi, Bluetooth, or cellular networks.

## Features

- **Robust Error Correction**: Uses RaptorQ (Fountain Codes) to ensure successful reconstruction even if frames are skipped, duplicated, or scanned out of order.
- **Compression**: Payloads are compressed with Zstandard prior to encoding to minimize data size.
- **Cross-platform Native Core**: Core logic (compression, framing, encoding, decoding) is written in Rust (`shared-core`) and used natively by both the Desktop and Android apps.

## Project Structure

- `shared-core/`: A Rust library that implements the QR frame protocol, Zstd compression, and RaptorQ encoding/decoding. Exports C-FFI / JNI bindings for Android.
- `desktop/`: A Rust application using `egui` that allows you to select a file, encodes it into a stream of QR code matrices via `fast_qr`, and displays them at high FPS.
- `android/`: An Android application built with Kotlin and Jetpack Compose. Scans the stream of QR codes using CameraX and feeds the payloads into the `shared-core` JNI decoder.

## Building the Project

### Prerequisites
- **Rust**: Ensure Rust is installed with a working C++ build toolchain (MSVC on Windows or GNU).
- **Android Studio / SDK**: Required to build the Android application.

### Building the Shared Core (Android JNI)
You must compile the `shared-core` for Android NDK targets:
```bash
cd shared-core
cargo build --target aarch64-linux-android --release
# Copy the resulting .so file to android/app/src/main/jniLibs/arm64-v8a/
```

### Building the Desktop App
```bash
cd desktop
cargo run --release
```

### Building the Android App
Open the `android/` folder in Android Studio, sync Gradle, and deploy to your physical Android device.
