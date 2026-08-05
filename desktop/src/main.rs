use crossbeam_channel::{unbounded, Receiver, Sender};
use eframe::egui;
use fast_qr::qr::QRBuilder;
use rfd::FileDialog;
use shared_core::encoder::QrEncoder;
use shared_core::protocol::Frame;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 800.0])
            .with_min_inner_size([600.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "QR Code File Transfer",
        options,
        Box::new(|cc| Box::new(App::new(cc))),
    )
}

enum AppState {
    Idle,
    Transferring,
}

struct App {
    state: AppState,
    selected_file: Option<PathBuf>,
    fps: f32,
    chunk_size: u16,
    qr_texture: Option<egui::TextureHandle>,
    
    // Communication with encoder thread
    command_tx: Sender<WorkerCommand>,
    frame_rx: Receiver<egui::ColorImage>,
}

enum WorkerCommand {
    Start(PathBuf, u16, f32), // file, chunk_size, fps
    Stop,
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (command_tx, command_rx) = unbounded();
        let (frame_tx, frame_rx) = unbounded();

        // Background worker thread for encoding QR codes
        thread::spawn(move || {
            worker_thread(command_rx, frame_tx);
        });

        Self {
            state: AppState::Idle,
            selected_file: None,
            fps: 15.0,
            chunk_size: 512,
            qr_texture: None,
            command_tx,
            frame_rx,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Try to receive a new frame if we are transferring
        if matches!(self.state, AppState::Transferring) {
            if let Ok(color_image) = self.frame_rx.try_recv() {
                self.qr_texture = Some(ctx.load_texture(
                    "qr_frame",
                    color_image,
                    egui::TextureOptions::NEAREST,
                ));
                ctx.request_repaint(); // Continuously request repaint while receiving frames
            } else {
                // Keep requesting repaint so the UI doesn't sleep while waiting for the next frame
                ctx.request_repaint_after(Duration::from_secs_f32(1.0 / self.fps));
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.state {
                AppState::Idle => {
                    ui.heading("QR Code File Transfer");
                    ui.add_space(20.0);

                    if ui.button("Select File").clicked() {
                        if let Some(path) = FileDialog::new().pick_file() {
                            self.selected_file = Some(path);
                        }
                    }

                    if let Some(ref path) = self.selected_file {
                        ui.label(format!("Selected: {}", path.display()));
                        
                        ui.add_space(10.0);
                        ui.add(egui::Slider::new(&mut self.fps, 1.0..=60.0).text("FPS"));
                        ui.add(egui::Slider::new(&mut self.chunk_size, 128..=1500).text("Chunk Size (bytes)"));
                        ui.add_space(20.0);

                        if ui.button("Start Transfer").clicked() {
                            self.state = AppState::Transferring;
                            self.command_tx.send(WorkerCommand::Start(
                                path.clone(),
                                self.chunk_size,
                                self.fps,
                            )).unwrap();
                        }
                    }
                }
                AppState::Transferring => {
                    ui.horizontal(|ui| {
                        ui.heading("Transferring...");
                        if ui.button("Stop").clicked() {
                            self.state = AppState::Idle;
                            self.qr_texture = None;
                            self.command_tx.send(WorkerCommand::Stop).unwrap();
                        }
                    });

                    ui.add_space(10.0);

                    if let Some(texture) = &self.qr_texture {
                        let available_size = ui.available_size();
                        let size = available_size.x.min(available_size.y);
                        ui.image((texture.id(), egui::vec2(size, size)));
                    } else {
                        ui.spinner();
                        ui.label("Generating first frame...");
                    }
                }
            }
        });
    }
}

fn worker_thread(rx: Receiver<WorkerCommand>, tx: Sender<egui::ColorImage>) {
    let mut current_encoder: Option<QrEncoder> = None;
    let mut fps = 1.0;
    
    // We send metadata first, then wait a bit, then loop data frames
    let mut sent_metadata = false;

    loop {
        // Check for new commands
        if let Ok(cmd) = rx.try_recv() {
            match cmd {
                WorkerCommand::Start(path, chunk_size, new_fps) => {
                    if let Ok(data) = fs::read(&path) {
                        let filename = path.file_name().unwrap().to_string_lossy().into_owned();
                        // Initialize encoder
                        if let Ok(encoder) = QrEncoder::new(&data, filename, chunk_size) {
                            current_encoder = Some(encoder);
                            fps = new_fps;
                            sent_metadata = false;
                        }
                    }
                }
                WorkerCommand::Stop => {
                    current_encoder = None;
                }
            }
        }

        if let Some(encoder) = &mut current_encoder {
            let frame = if !sent_metadata {
                sent_metadata = true;
                encoder.get_metadata_frame()
            } else {
                encoder.next_data_frame()
            };

            let frame_bytes = frame.encode();
            
            // Build QR Code
            if let Ok(qr) = QRBuilder::new(frame_bytes).build() {
                // Convert to color image
                let module_size = 8;
                let size = qr.size * module_size;
                let mut pixels = vec![egui::Color32::WHITE; size * size];
                
                for y in 0..qr.size {
                    for x in 0..qr.size {
                        if qr.get_module(x, y) {
                            // fill an 8x8 block
                            for dy in 0..module_size {
                                for dx in 0..module_size {
                                    let py = y * module_size + dy;
                                    let px = x * module_size + dx;
                                    pixels[py * size + px] = egui::Color32::BLACK;
                                }
                            }
                        }
                    }
                }
                
                let image = egui::ColorImage {
                    size: [size, size],
                    pixels,
                };
                
                // Ignore send errors (happens if UI disconnects)
                let _ = tx.send(image);
            }
            
            // Sleep to maintain FPS
            thread::sleep(Duration::from_secs_f32(1.0 / fps));
        } else {
            // Idle, block until command
            if let Ok(cmd) = rx.recv() {
                match cmd {
                    WorkerCommand::Start(path, chunk_size, new_fps) => {
                        if let Ok(data) = fs::read(&path) {
                            let filename = path.file_name().unwrap().to_string_lossy().into_owned();
                            if let Ok(encoder) = QrEncoder::new(&data, filename, chunk_size) {
                                current_encoder = Some(encoder);
                                fps = new_fps;
                                sent_metadata = false;
                            }
                        }
                    }
                    WorkerCommand::Stop => {}
                }
            }
        }
    }
}
