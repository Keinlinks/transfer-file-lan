use std::sync::mpsc::Sender;

use eframe::egui::{self, Align, Layout};
use rfd::FileDialog;
use crate::structs::MetadataSeriable;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ConfirmWindow {
    metadata: MetadataSeriable,
    download_dir: String,
    #[serde(skip)]
    tx:Option<Sender<(String,String)>>,
}


impl Default for ConfirmWindow{
    fn default() -> Self {
        Self {
            metadata: MetadataSeriable {
                len: 0,
                is_dir: false,
                is_file: false,
                name: String::new(),
            },
            download_dir: String::new(),
            tx: None
        }
    }
}

impl ConfirmWindow{
    pub fn new (_: &eframe::CreationContext<'_>, metadata: MetadataSeriable, download_dir: String,tx: Option<Sender<(String, String)>>) -> Self {
        Self {
            metadata,
            download_dir,
            tx
        }
    }

    pub fn close_confirm_window(&mut self) {
        
    }
}

impl eframe::App for ConfirmWindow {
    fn update(&mut self, ctx: &eframe::egui::Context, _: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                        if ui.button("Select download folder").clicked() {
                            let dir = FileDialog::new().pick_folder();
                            if let Some(dir) = dir {
                                self.download_dir = dir.display().to_string();
                                if let Some(tx) = &self.tx {
                                    if self.download_dir.is_empty(){
                                        self.download_dir = "C:\\".to_string();
                                    }
                                    tx.send(("path".to_string(), self.download_dir.clone())).unwrap();
                                }
                            };  
                        }
                    ui.add_space(12.0);
                ui.label("Download folder: ".to_string() + &self.download_dir);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Name: ".to_string() + &self.metadata.name);
            let size = bytes_to_mb(self.metadata.len).to_string();
            ui.label("Size: ".to_string() + &size + " Mb");
            ui.add_space(25.0);
            ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                if ui.button("Accept").clicked() {
                    if let Some(tx) = &self.tx {
                        tx.send(("yes".to_string(), self.download_dir.clone())).unwrap();
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            });
            ui.with_layout(Layout::bottom_up(Align::RIGHT), |ui| {
                if ui.button("Cancel").clicked() {
                    if let Some(tx) = &self.tx {
                        tx.send(("no".to_string(), self.download_dir.clone())).unwrap();
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            });
            
        });
        
    }
}

fn bytes_to_mb(bytes: u64) -> f64 {
    const BYTES_IN_MB: u64 = 1_048_576;
    let mb = bytes as f64 / BYTES_IN_MB as f64;
    (mb * 100.0).round() / 100.0
}