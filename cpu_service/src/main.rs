use common::CPUUsage;
use std::thread;
use std::time::Duration;
use sysinfo::System;

fn main() {
    println!("CPU Usage monitor started");
    let mut sys = System::new();
    // Open serial port to send data to
    let port = std::env::args().skip(1).next().unwrap();
    let mut serial = serialport::new(port, 115_200).timeout(Duration::from_millis(100)).open().expect("Could not open serial port");
    loop {
        sys.refresh_cpu_usage();
        let cpu_usage_data: Vec<CPUUsage> = sys
            .cpus()
            .iter()
            .map(|cpu| CPUUsage {
                id: cpu
                    .name()
                    .chars()
                    .skip(3)
                    .collect::<String>()
                    .parse()
                    .unwrap(),
                usage: cpu.cpu_usage(),
            })
            .collect();
        let serialized = postcard::to_allocvec(&cpu_usage_data).unwrap();
        println!("Sending {} bytes of cpu usage data", serialized.len());
        let bytes_written = serial.write(&serialized).unwrap();
        assert_eq!(bytes_written, serialized.len());
        std::thread::sleep(Duration::from_millis(300));
    }
}

// TODO: Create a trait for sending the CPU usage data to the receiver over any transport.
