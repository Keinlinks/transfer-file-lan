use std::{fs::File, io::{Read, Write}, net::{Shutdown, TcpStream}, sync::{mpsc, Arc, Mutex}, thread};

use eframe::egui::{self, Widget};
use crate::structs::MetadataSeriable;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct DownloadWindow {
    progress: Arc<Mutex<f32>>,
    download_dir: String,
    metadata: MetadataSeriable,
    debug: Arc<Mutex<String>>,
}


impl Default for DownloadWindow{
    fn default() -> Self {
        Self {
            metadata: MetadataSeriable {
                len: 0,
                is_dir: false,
                is_file: false,
                name: String::new(),
            },
            download_dir: String::new(),
            progress: Arc::new(Mutex::new(0.0)),
            debug: Arc::new(Mutex::new(String::new())),
        }
    }
}

impl DownloadWindow{
    pub fn new (_: &eframe::CreationContext<'_>, metadata: MetadataSeriable, download_dir: String, mut stream: TcpStream) -> Self {
        let (tx, rx) = mpsc::channel();
        let file_path = download_dir.clone() + "\\" + &metadata.name;
        let debug = Arc::new(Mutex::new(String::new()));
        let debug_clone = Arc::clone(&debug);
        thread::spawn(move || {
            let file = File::create(&file_path);
            match file {
                Ok(mut file) => {
                    let mut total_bytes_read = 0;
                    let mut buffer = vec![0; 8192];
                    loop {
                        match stream.read(&mut buffer) {
                            Ok(0) => {
                                stream.shutdown(Shutdown::Both).unwrap();
                                let mut debug_lock = debug_clone.lock().unwrap();
                                *debug_lock = "Archivo recibido".to_string();
                                break;
                            },
                            Ok(bytes_read) => {
                                file.write_all(&buffer[..bytes_read ]).unwrap();
                                total_bytes_read += bytes_read;
                                tx.send(total_bytes_read as f32).unwrap();
                            },
                            Err(e) => {
                                eprintln!("Error al leer del stream: {}", e);
                                break;
                            },
                        }
                    }
                },
                Err(e) => {
                    println!("Error: {}", e);   
                }
            }
        });
        let progress = Arc::new(Mutex::new(0.0));
        let progress_clone = Arc::clone(&progress);
        let total_bytes = metadata.len as f32;
        thread::spawn(move ||{
            for received in rx {
                let mut progress_lock = progress_clone.lock().unwrap();
                *progress_lock =  received/ total_bytes;
             }
        });
        Self {
            metadata,
            download_dir,
            progress,
            debug
        }
    }
}

impl eframe::App for DownloadWindow {
    fn update(&mut self, ctx: &eframe::egui::Context, _: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.label("Descargando a: ".to_string() + &self.download_dir);
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Nombre del archivo: ".to_string() + &self.metadata.name);
            ui.label("Tamaño: ".to_string() + &self.metadata.len.to_string());
            ui.add_space(25.0);
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.label(self.debug.lock().unwrap().to_string());
                let progress = self.progress.lock().unwrap();
                egui::ProgressBar::new(*progress).show_percentage().ui(ui);
                ctx.request_repaint();
           });
        });
       
        
    }
}