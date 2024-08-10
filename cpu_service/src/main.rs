use common::CPUUsage;
use std::thread;
use std::time::Duration;
use sysinfo::System;

fn main() {
    println!("CPU Usage monitor started");
    let mut sys = System::new();
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
        println!("{:?}", cpu_usage_data);
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    }
}

// TODO: Create a trait for sending the CPU usage data to the receiver over any transport.
