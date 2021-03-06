
use std::thread;

use rocket::http::Status;
use sys_info::{os_type, os_release, cpu_num, cpu_speed, disk_info, mem_info, loadavg};
use systemstat::{System, Platform, Duration};
extern crate systemstat;

#[get("/sys/mem")]
pub async fn get_ram() -> (Status, String) {
    match mem_info() {
        Ok(mem) => return (Status::Ok, format!("{}/{}", mem.free, mem.total)),
        Err(_) => return (Status::BadRequest, "failed to get ram".to_string())
    }
}

#[get("/sys/disk")]
pub async fn get_disk() -> (Status, String) {
    match disk_info() {
        Ok(disk) => return (Status::Ok, format!("{}/{}", disk.free, disk.total)),
        Err(_) => return (Status::BadRequest, "failed to get disk".to_string())
    }
}

#[get("/sys/cpuspeed")]
pub async fn get_cpu_speed() -> (Status, String) {
    match cpu_speed() {
        Ok(cpuspeed) => return (Status::Ok, cpuspeed.to_string()),
        Err(_) => return (Status::BadRequest, "failed to get cpu speed".to_string())
    }
}
/// DOES NOT WORK IN WSL
#[get("/sys/cpuinfo")]
pub async fn get_cpu_info() -> (Status, String) {
    // TODO: get CPU info without extra dependencies
    (Status::Ok, "testcpu".to_string())
    // match cpuid::identify() {
    //     Ok(cpuinfo) => return (Status::Ok, format!("{} {}", cpuinfo.vendor, cpuinfo.codename)),
    //     Err(_) => return (Status::BadRequest, "failed to get cpu info".to_string())
    // }
}
/// This handler will always take 1s+ to respond
#[get("/sys/cpuutil")]
pub async fn get_utilization() -> (Status, String) {
    let sys = System::new();
    match sys.cpu_load_aggregate() {
        Ok(load) => {
            thread::sleep(Duration::from_secs(1));
            return (Status::Ok, load.done().unwrap().user.to_string())
        },
        Err(_) => return (Status::BadRequest, "failed to get cpu info".to_string())
    }
}

#[get("/sys/osinfo")]
pub async fn get_os_info() -> (Status, String) {
    match os_release() {
        Ok(release) => {
            match os_type() {
                Ok(ostype) => (Status::Ok, format!("{} {}", ostype, release)),
                Err(_) => return (Status::BadRequest, "failed to get os info".to_string())
            }
        }
        Err(_) => return (Status::BadRequest, "failed to get os info".to_string())
    }
}

#[get("/sys/uptime")]
pub async fn get_uptime() -> (Status, String) {
    let sys = System::new();
    match sys.uptime() {
        Ok(uptime) => return (Status::Ok, format!("{}", uptime.as_secs_f64().to_string())),
        Err(_) => return (Status::BadRequest, "failed to get cpu info".to_string())
    }
}