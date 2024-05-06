use daemonize::Daemonize;
use std::{fs::{File, OpenOptions}, io, process::Command};

const NAME: &str = "php-job-runner";
const WORKING_DIRECTORY: &str = "/var/www/html";
const COMMAND: &str = "php artisan schedule:run";

fn main() -> io::Result<()> {
    let stdout = File::create("/var/log/php-job-daemon.log")?;
    let stderr = File::create("/var/log/php-job-daemon.error.log")?;
    let daemon = Daemonize::new()
        .pid_file("/tmp/php-job.pid")
        .chown_pid_file(true)
        .working_directory(WORKING_DIRECTORY) 
        .user("root")
        .group("root")
        .group(2) 
        .umask(0o777) 
        .stdout(stdout) 
        .stderr(stderr) 
        .privileged_action(|| "Executed before drop privileges");

    match daemon.start() {
        Ok(_) => {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(30));
                run()?;
            }
        }
        Err(e) => eprintln!("Failed to start daemon: {}", e),
    }

    Ok(())
}

fn run() -> io::Result<()> {
    let output = Command::new("php")
        .arg("artisan")
        .arg("schedule:run").output()?;
    if !output.status.success() {
        eprintln!(
            "Failed to execute command {}: {}",
            COMMAND,
            String::from_utf8_lossy(&output.stderr)
        );

        return Err(io::Error::new(io::ErrorKind::Other, "Command failed"));
    }

    println!("Succeeded");

    Ok(())
}
