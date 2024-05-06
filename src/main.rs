use daemonize::Daemonize;
use std::{fs::File, io, process::Command};

const NAME: &str = "php-job-runner";
const WORKING_DIRECTORY: &str = "/var/www/html";
const COMMAND: &str = "php artisan schedule:run";

fn main() -> io::Result<()> {
    let stdout = File::create(format!("/var/log/{NAME}.log"))?;
    let stderr = File::create(format!("/var/log/{NAME}.error.log"))?;

    let daemonize = Daemonize::new()
        .pid_file("/tmp/{NAME}.pid") // Every method except `new` and `start`
        .chown_pid_file(true) // is optional, see `Daemonize` documentation
        .working_directory(WORKING_DIRECTORY) // for default behaviour.
        .user("root")
        .group("root") // Group name
        .group(2) // or group id.
        .umask(0o777) // Set umask, `0o027` by default.
        .stdout(stdout) // Redirect stdout to `/tmp/daemon.out`.
        .stderr(stderr) // Redirect stderr to `/tmp/daemon.err`.
        .privileged_action(|| "Executed before drop privileges");

    match daemonize.start() {
        Ok(_) => {
            std::thread::sleep(std::time::Duration::from_secs(300));

            loop {
                run()?;
                std::thread::sleep(std::time::Duration::from_secs(60));
            }
        }
        Err(e) => eprintln!("Failed to start daemon: {}", e),
    }

    Ok(())
}

fn run() -> io::Result<()> {
    if let Err(err) = std::env::set_current_dir(WORKING_DIRECTORY) {
        eprintln!(
            "Failed to change directory to {}: {}",
            WORKING_DIRECTORY, err
        );

        return Err(err);
    }

    let output = Command::new(COMMAND).output()?;
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
