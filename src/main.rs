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
    command: String,
}

const SECONDS_IN_A_MINUTE: u16 = 60;
const SECONDS_IN_A_HOUR: u16 = 60 * 60;

struct Retry {
    command: String,
    history: Vec<u64>,
    max_retries: u8,
    timespan: u16,
    restart_delay: u8,
    restart_name: String,
}

fn main() {
    let arguments = Arguments::parse();

    let mut max_retries = 4;
    let mut timespan = SECONDS_IN_A_MINUTE;
    let mut restart_name = String::from("minute");

    let Arguments { per_minute, per_hour, .. } = arguments;
    if per_minute > 0 {
        max_retries = per_minute;
    } else if per_hour > 0 {
        max_retries = per_hour;
        timespan = SECONDS_IN_A_HOUR;
        restart_name = String::from("hour");
    }

    let Arguments { command, delay: restart_delay, .. } = arguments;
    let retry: Retry = Retry {
        max_retries,
        timespan,
        restart_name,
        command,
        restart_delay,
        history: Vec::new(),
    };

    run_command(retry)
}

fn get_now() -> u64 {
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
}

fn push_history(mut history: Vec<u64>, timespan: u16) -> Vec<u64> {
    history.push(get_now() + u64::from(timespan));

    return history.to_owned()
}

fn update_history(mut history: Vec<u64>) -> Vec<u64> {
    let now = get_now();
    let clear_times: Vec<u64> = history
        .iter()
        .filter(|&time| time <= &now)
        .map(|&time| time)
        .collect();

    for time in clear_times {
        history.retain(|&h| h != time);
    }

    return history.to_owned();
}

fn check_history(history: &Vec<u64>, max_retries: u8) -> bool {
    return history.len().lt(&usize::from(max_retries))
}

fn spawn_process(retry: &Retry) -> Result<Child, std::io::Error> {
    let mut command_parts: Vec<String> = retry
        .command
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    let mut command: Command = Command::new(&command_parts[0]);
    command_parts.remove(0);
    command.args(command_parts);

    match command.spawn() {
        Ok(child) => Ok(child),
        Err(err) => Err(err),
    }
}

fn run_command(mut retry: Retry) {
    let mut process = spawn_process(&retry)
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
        let pushed_history = push_history(retry.history, retry.timespan);
        let updated_history = update_history(pushed_history);
        if check_history(&updated_history, retry.max_retries) {
            retry.history = updated_history;
            println!("Restarting...");
            restart(retry);
        } else {
            println!("The process has crashed more than {} times in the past {}, stop restarting\n", retry.max_retries, retry.restart_name);
        }
    }
}

fn restart(retry: Retry) {
    sleep(Duration::from_secs(u64::from(retry.restart_delay)));

    run_command(retry);
}
