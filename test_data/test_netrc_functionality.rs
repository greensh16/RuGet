use std::fs::File;
use std::io::BufReader;
use netrc::Netrc;

fn main() {
    println!("Testing .netrc parsing functionality...");
    
    // Test parsing our test netrc file
    match File::open("test_data/test_netrc") {
        Ok(file) => {
            match Netrc::parse(BufReader::new(file)) {
                Ok(netrc) => {
                    println!("Successfully parsed .netrc file");
                    println!("Found {} machine entries", netrc.hosts.len());
                    
                    for (host, machine) in &netrc.hosts {
                        println!("Host: {}", host);
                        println!("  Login: {}", machine.login);
                        println!("  Password: {}", if machine.password.is_some() { "[PRESENT]" } else { "[NONE]" });
                    }
                    
                    // Test the lookup functionality
                    if let Some((_, machine)) = netrc.hosts.iter().find(|(h, _)| h == "httpbin.org") {
                        if !machine.login.is_empty() {
                            if let Some(password) = &machine.password {
                                if !password.is_empty() {
                                    println!("\n✅ Found credentials for httpbin.org:");
                                    println!("   Username: {}", machine.login);
                                    println!("   Password: [REDACTED]");
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Error parsing .netrc file: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Error opening .netrc file: {}", e);
        }
    }
}
