use std::{net::IpAddr, process::exit, time::Duration};

use icmp::{self, EchoRequest, EchoResult, IpVer, lookup_host};

fn main() {
    // TODO: clap

    let ver = IpVer::V4;
    let host = "google.com";
    // let host = "8.8.8.8";
    let addr: IpAddr = host
        .parse()
        .or_else(|_| {
            let res = lookup_host(host, ver);
            #[cfg(debug_assertions)]
            if let Ok(addr) = res {
                println!("Host {} resolved to {}", host, addr);
            }
            res
        })
        .unwrap_or_else(|_e| {
            eprintln!("Failed to resolve hostname '{}'", host);
            exit(2);
        });

    let request_data = b"hello";
    let timeout = Duration::from_secs(10);
    let req = EchoRequest {
        addr,
        data: request_data,
        timeout,
    };

    let mut sess = icmp::echo_session(req).unwrap_or_else(|e| {
        eprintln!("Failed to start echo session: {:?}", e);
        exit(2);
    });

    let count = 2;
    let sleep_duration = Duration::from_secs(1);
    let flood = false;
    println!(
        "Pinging {} with {} bytes of data:",
        host,
        request_data.len()
    );
    if flood {
        // Collect to not slow down flood with stdout writing.
        let mut results = Vec::with_capacity(count);
        icmp::echo(&mut sess, count).for_each(|res| results.push(res));
        for res in results {
            print_res(&res);
        }
    } else {
        icmp::echo(&mut sess, count).for_each(|res| {
            print_res(&res);
            std::thread::sleep(sleep_duration);
        });
    }
}

fn print_res(res: &EchoResult) {
    match res {
        Ok(reply) => {
            let n_bytes = reply.data.len();
            println!(
                "Reply from {}: bytes={} time={}ms TTL={}",
                reply.addr,
                n_bytes,
                reply.rtt.as_millis(),
                reply.ttl
            );
            println!(
                "\tReceived message: {:?}",
                String::from_utf8_lossy(&reply.data)
            );
        }
        Err(err) => {
            println!("Error: {:?}", err) // FIXME
        }
    }
}
