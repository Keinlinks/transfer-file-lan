use serde::{Serialize, Deserialize};
use local_ip_address::local_ip;
use gethostname::gethostname;
use mdns_sd::{ServiceDaemon, ServiceInfo};
#[derive(Serialize, Deserialize, Debug)]
struct Device {
    id: i32,
    name: String,
    ip: String,
    port: u16,
}


pub fn start_mdns_server(service_type: &str, instance_name: &str, port: u16,tcp_port: u16) -> mdns_sd::ServiceDaemon {
    let mdns = ServiceDaemon::new().expect("Failed to create daemon");
    let my_local_ip = local_ip().unwrap();

    let ip = my_local_ip;
    let host_name = gethostname().into_string().unwrap() + ".local.";

    let properties = [("name", gethostname().into_string().unwrap()),("tcp_port", tcp_port.to_string())];

    let my_service = ServiceInfo::new(
        service_type,
        instance_name,
        &host_name,
        ip.to_string(),
        port,
        &properties[..],
    ).expect("Failed to create service info");
    mdns.register(my_service).expect("Failed to register our service");

    mdns
}
pub fn mdns_client_side(service_type: &str) -> mdns_sd::Receiver<mdns_sd::ServiceEvent> {
    let mdns = ServiceDaemon::new().expect("Failed to create daemon");

    mdns.browse(service_type).expect("Failed to browse")
}