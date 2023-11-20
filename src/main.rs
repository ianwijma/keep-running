use std::process::{Child, Command};
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use clap::{arg, Parser};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    // The amount of times we retry in a span of a minute, before we stop retrying
    #[arg(long, default_value_t = 0, conflicts_with = "per_hour")]
    per_minute: u8,
    // The amount of times we retry in a span of a hour, before we stop retrying
    #[arg(long, default_value_t = 0, conflicts_with = "per_minute")]
    per_hour: u8,
    // The amount of seconds we want to delay the restart with.
    #[arg(long, default_value_t = 0)]
    delay: u8,

    #[arg()]
    command: String
}

const SECONDS_IN_A_MINUTE: u16 = 60;
const SECONDS_IN_A_HOUR: u16 = 60 * 60;

struct Retry {
    command: String,
    history: Vec<u64>,
    max_retries: u8,
    timespan: u16,
    restart_delay: u8,
    restart_name: String
}

fn main() {
    let arguments = Arguments::parse();

    let mut retry: Retry = Retry {
        command: arguments.command,
        history: Vec::new(),
        max_retries: 4,
        timespan: SECONDS_IN_A_MINUTE,
        restart_delay: arguments.delay,
        restart_name: "minute".to_string(),
    };

    if arguments.per_minute > 0 {
        retry.max_retries = arguments.per_minute;
    } else if arguments.per_hour > 0 {
        retry.max_retries = arguments.per_hour;
        retry.timespan = SECONDS_IN_A_HOUR;
        retry.restart_name = "hour".to_string();
    }

    run_command(&mut retry)
}

fn get_now() -> u64 {
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
}

fn push_history(retry: &mut Retry) {
    retry.history.push(get_now() + u64::from(retry.timespan));
}

fn update_history(retry: &mut Retry) {
    let now = get_now();
    let clear_times: Vec<u64> = retry
        .history
        .iter()
        .filter(|&time| time <= &now)
        .map(|&time| time)
        .collect();

    for time in clear_times {
        retry.history.retain(|&h| h != time);
    }
}

fn check_history(retry: &Retry) -> bool {
    return retry.history.len().lt(&usize::from(retry.max_retries))
}

fn spawn_process(retry: &Retry) -> std::io::Result<Child> {
    let mut command_parts: Vec<String> = retry
        .command
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    let mut command: Command = Command::new(&command_parts[0]);
    command_parts.remove(0);
    command.args(command_parts);

    return command.spawn();
}

fn run_command(retry: &mut Retry) {
    let mut process = spawn_process(retry)
        .expect("Process failed on startup");

    let exit_code = match process.wait() {
        Ok(exit_code) => exit_code,
        Err(err) => {
            eprintln!("Failed to wait for process: {}", err);
            return;
        }
    };

    if exit_code.success() {
        println!("Exit code: {}", exit_code);
    } else {
        println!("[CRASH] exit code: {}", exit_code);
        push_history(retry);
        update_history(retry);
        if check_history(retry) {
            println!("Restarting...");
            restart(retry);
        } else {
            print!("The process has crashed more then {} times in the past {}, stop restarting\n", retry.max_retries, retry.restart_name);
        }
    }
}

fn restart(retry: &mut Retry) {
    sleep(Duration::from_secs(u64::from(retry.restart_delay)));

    run_command(retry);
}
