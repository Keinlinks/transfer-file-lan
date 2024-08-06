#![windows_subsystem = "windows"]
use std::env;
use std::fs::metadata;
use std::io::Write;
use std::net::TcpStream;
use std::path::Path;
use std::process::{exit, Command};
use std::sync::{Arc, Mutex};
use std::{io::Read, net::TcpListener, sync::mpsc, thread};
use tray_item::{IconSource, TrayItem};
use local_ip_address::local_ip;
use tfer_windows_service::{mdns_module, ConfirmWindow, DownloadWindow, SendWindow};
use tfer_windows_service::structs::MetadataSeriable;
struct Message{
    message: String,
    buffer: Vec<u8>,
    stream: Option<TcpStream>,
}

const TCP_PORT : u16 = 5205;
const MDNS_PORT : u16 = 5200;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    mdns_module::start_mdns_server("_tfer._udp.local.", "tfer_instance", MDNS_PORT, TCP_PORT);

    if args.len() == 1 {
        open_tray_mode();
    }
    else{
        open_gui_mode(args[1].clone())
    }
    
}

fn open_gui_mode(path_file: String){
    let metadata = metadata(&path_file).unwrap();
    let metadata = MetadataSeriable {
        len: metadata.len(),
        is_dir: metadata.is_dir(),
        is_file: metadata.is_file(),
        name: Path::new(&path_file).file_name().unwrap().to_str().unwrap().to_string(),
    };
    let native_options = eframe::NativeOptions {
        run_and_return: true,
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([600.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_max_inner_size([900.0, 300.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    let _ =eframe::run_native(
        "tfer_app",
        native_options,
        Box::new(|cc| Ok(Box::new(SendWindow::new(cc,metadata,path_file)))),
    );
}

fn open_tray_mode(){
    let mut tray = TrayItem::new(
        "Tfer",
        IconSource::Resource("name-of-icon-in-rc-file"),
    )
    .unwrap();

    tray.inner_mut().add_separator().unwrap();


    let (tx, rx) = mpsc::sync_channel(1);
    
    tray.inner_mut().add_separator().unwrap();

    let quit_tx = tx.clone();
    tray.add_menu_item("Salir", move || {
        let message = Message{message: "quit".to_string(), buffer: Vec::new(), stream: None};
        quit_tx.send(message).unwrap();
    })
    .unwrap();

    thread::spawn(move ||{
        let listener = open_tcp_server(TCP_PORT).unwrap();

        loop {
            for stream in listener.incoming() {
                match stream{
                    Ok(mut stream) => {
                        let mut buffer = [0; 1024];
                        match stream.read(&mut buffer) {
                            Ok(_n) => {
                                //deberia llegar la metadata
                                let message = Message{message: "open_confirmation_window".to_string(), buffer: buffer.to_vec(), stream: Some(stream)};
                                tx.send(message).unwrap();
                                
                                
                            }
                            Err(e) => {
                                println!("Error: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
        }
    });
    loop {
        if let Ok(Message{message, buffer,stream}) = rx.recv() {
            if message == "quit" {
                println!("Quit");
                break;
            }
            if message == "open_confirmation_window"{
                if let Some(mut stream) = stream {
                    let metadata = bincode::deserialize::<MetadataSeriable>(&buffer[..]).unwrap();
                    let metadata_clone = MetadataSeriable {
                        is_dir: metadata.is_dir,
                        is_file: metadata.is_file,
                        len: metadata.len,
                        name: metadata.name.clone(),
                    };
                    let (answer, download_dir) = open_confirmation_window(metadata_clone).unwrap();
                    if answer == "yes"{
                        let buffer = bincode::serialize(&answer).unwrap();
                        stream.write_all(&buffer).unwrap();
                        download_process(stream.try_clone().unwrap(), download_dir, metadata);
                    }
                    else{
                        stream.shutdown(std::net::Shutdown::Both).unwrap();
                    }
                };
                
            }
        }
    }
}



pub fn open_tcp_server(port:u16)-> Result<TcpListener, std::io::Error> {
    let my_local_ip = local_ip().unwrap();
    TcpListener::bind((my_local_ip, port))
}


fn open_confirmation_window(metadata: MetadataSeriable) -> Result<(String,String), eframe::Error>{
    let (tx, rx):(mpsc::Sender<(String,String)>, mpsc::Receiver<(String,String)>) = mpsc::channel();
    let answer = Arc::new(Mutex::new(String::new()));
    let download_dir = Arc::new(Mutex::new(String::new()));

    let answer_clone = Arc::clone(&answer);
    let download_dir_clone = Arc::clone(&download_dir);

    thread::spawn(move ||{
        for received in rx.iter(){
            if received.0 == "yes" || received.0 == "no"{
                let mut answer_ = answer_clone.lock().unwrap();
                *answer_ = received.0;
                break;
            }
            else{
                let mut download_dir_ = download_dir_clone.lock().unwrap();
                *download_dir_ = received.1;
            }
            
        }
    });
    let native_options = eframe::NativeOptions {
        run_and_return: true,
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([600.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_max_inner_size([400.0, 300.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    let _ =eframe::run_native(
        "tfer_app",
        native_options,
        Box::new(|cc| Ok(Box::new(ConfirmWindow::new(cc,metadata, String::new(), Some(tx))))),
    );
    let answer_cloned = {
        let answer_locked = answer.lock().unwrap();
        answer_locked.clone()
    };

    let download_dir_cloned = {
        let download_dir_locked = download_dir.lock().unwrap();
        download_dir_locked.clone()
    };
    Ok((answer_cloned,download_dir_cloned))
}

fn download_process(stream: TcpStream, download_dir: String,metadata: MetadataSeriable){
    let native_options = eframe::NativeOptions {
        run_and_return: true,
        centered: true,
        persist_window: false,
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([600.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_max_inner_size([900.0, 300.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    let _ =eframe::run_native(
        "tfer_app",
        native_options,
        Box::new(|cc| Ok(Box::new(DownloadWindow::new(cc,metadata, download_dir,stream)))),
    );
    restart().unwrap();

}

fn restart() -> std::io::Result<()> {
    let current_exe = env::current_exe()?;

    Command::new(current_exe)
        .spawn()?;

    exit(0);
}