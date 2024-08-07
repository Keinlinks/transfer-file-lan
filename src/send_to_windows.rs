use std::{fs::File, io::{Read, Write}, net::{IpAddr, Shutdown, TcpStream}, sync::{mpsc, Arc, Mutex}, thread};

use eframe::egui::{self, Widget};
use local_ip_address::local_ip;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use crate::structs::{MetadataSeriable, Device};
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct SendWindow {
    progress: Arc<Mutex<f32>>,
    metadata: MetadataSeriable,
    debug: Arc<Mutex<String>>,
    devices: Arc<Mutex<Vec<Device>>>,
    file_path: String,
    own_ip: IpAddr,
    sending: bool,
}
const TCP_PORT: u16 = 5205;

impl Default for SendWindow{
    fn default() -> Self {
        Self {
            metadata: MetadataSeriable {
                len: 0,
                is_dir: false,
                is_file: false,
                name: String::new(),
            },
            progress: Arc::new(Mutex::new(0.0)),
            debug: Arc::new(Mutex::new(String::new())),
            devices: Arc::new(Mutex::new(Vec::new())),
            file_path: String::new(),
            own_ip: IpAddr::V4(std::net::Ipv4Addr::new(0,0,0,0)),
            sending: false,
        }
    }
}
fn mdns_client_side(service_type: &str) -> mdns_sd::Receiver<mdns_sd::ServiceEvent> {
    let mdns = ServiceDaemon::new().expect("Failed to create daemon");

    mdns.browse(service_type).expect("Failed to browse")
}
impl SendWindow{
    pub fn new (_: &eframe::CreationContext<'_>, metadata: MetadataSeriable,file_path: String) -> Self {
        let progress = Arc::new(Mutex::new(0.0));
        let debug = Arc::new(Mutex::new("Enviando".to_string()));

        let devices = Arc::new(Mutex::new(Vec::new()));
        let devices_clone = Arc::clone(&devices);
        thread::spawn(move || {
            let mdns_receiver = mdns_client_side("_tfer._udp.local.");
            let mut devices_ip_register:Vec<String> = Vec::new();
            while let Ok(event) = mdns_receiver.recv() {
                if let ServiceEvent::ServiceResolved(info) = event {
                    let ip = info.get_addresses_v4().into_iter().next().unwrap();
                    let mut data = devices_clone.lock().unwrap();
                    let contains = devices_ip_register.iter().any(|item| item == &ip.to_string());
                    if !contains {
                           data.push(Device {
                           id:1,
                           ip:ip.to_string(),
                           name: info.get_properties().get("name").unwrap().to_string(),
                           port: info.get_port()
                           });
                           devices_ip_register.push(ip.to_string());
                    }
                }
            }
        });
        
        Self {
            metadata,
            progress,
            debug,
            devices,
            file_path,
            own_ip: local_ip().unwrap(),
            sending: false,
        }
    }
    fn send_file(&self,ip: IpAddr,port: u16,file_path: String,metadata: &MetadataSeriable,progress_clone: Arc<Mutex<f32>>,debug_clone: Arc<Mutex<String>>){
        
        let total_bytes = metadata.len as f32;
        let (tx, rx): (mpsc::Sender<f32>, mpsc::Receiver<f32>) = mpsc::channel();
        let mut deb = debug_clone.lock().unwrap();
        *deb = "Sending...".to_string();
        let debug_clone = Arc::clone(&debug_clone);
        let metada_clone = MetadataSeriable {
            is_dir: metadata.is_dir,
            is_file: metadata.is_file,
            len: metadata.len,
            name: metadata.name.clone(),
        };
         thread::spawn(move || {
             let mut file = File::open(file_path).unwrap();
             let metadata_buf = bincode::serialize(&metada_clone).unwrap();
            let mut stream = TcpStream::connect((ip, port)).unwrap();
            stream.write_all(&metadata_buf).unwrap();

            let mut answer_buffer = vec![0; 8192];

            let r =stream.read(&mut answer_buffer).unwrap();
            
            let answer = bincode::deserialize::<String>(&answer_buffer[..r]).unwrap();
            if answer == "yes" {
                let mut buffer = vec![0; 8192];
                let mut total_bytes_sended = 0;
                loop{
                    let bytes_read = file.read(&mut buffer).unwrap();
                    if bytes_read == 0 {
                        let mut debug_lock = debug_clone.lock().unwrap();
                        *debug_lock = "File sent".to_string();
                         break;
                     };
                     let _ =stream.write_all(&buffer[..bytes_read]);
                     total_bytes_sended += bytes_read;
                    tx.send(total_bytes_sended as f32).unwrap();
                }
            }
            else{
                let mut debug_lock = debug_clone.lock().unwrap();
                *debug_lock = "Rejected the send".to_string();
                stream.shutdown(Shutdown::Both).unwrap();

            }
           
         });
        thread::spawn(move ||{
            for received in rx {
                let mut progress_lock = progress_clone.lock().unwrap();
                *progress_lock =  received/ total_bytes;
             }
        });
        
    }
}

impl eframe::App for SendWindow {
    fn update(&mut self, ctx: &eframe::egui::Context, _: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.label("Name: ".to_string() + &self.metadata.name);
            let size = bytes_to_mb(self.metadata.len).to_string();
            ui.label("Size: ".to_string() + &size + " Mb");
        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(12.0);
            let devices= Arc::clone(&self.devices);
            ui.label("Devices list");
            if devices.lock().unwrap().len() > 1{ ui.group(|ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {

                    ui.vertical(|ui| {
                        for device in devices.lock().unwrap().iter() {
                            let text = "Nombre: ".to_string() + &device.name + " - IP: " + &device.ip;
                            let ip_device = IpAddr::V4(device.ip.parse::<std::net::Ipv4Addr>().unwrap());
                            if self.own_ip != ip_device && ui.button(text).clicked() && !self.sending {
                                self.sending = true;
                                self.send_file(ip_device, TCP_PORT, self.file_path.clone(), &self.metadata, self.progress.clone(), self.debug.clone());
                            }
                        }
                    });
                });
            });
        }
        else{
            ui.label("Devices not found");
        }

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
fn bytes_to_mb(bytes: u64) -> f64 {
    const BYTES_IN_MB: u64 = 1_048_576;
    let mb = bytes as f64 / BYTES_IN_MB as f64;
    (mb * 100.0).round() / 100.0
}