use std::io;
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, ExitStatus};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
mod lib;

const BATCH_SIZE: usize = 100;

#[derive(Default)]
struct Statistics {
    fuzz_cases: AtomicUsize,
    num_crashes: AtomicUsize
}

fn fuzz(thr_id: usize, filename: &str, inp: &Vec<u8>) -> io::Result<ExitStatus> {
    // Write out the input to a temporary file
    let filepath = format!("./output/tmp_{}_{}", thr_id, &filename);
    std::fs::write(&filepath, inp).unwrap();

    let runner = Command::new("./exif").arg(&filepath).output()?;

    Ok(runner.status)
}

fn worker(thr_id: usize, stats: Arc<Statistics>) -> io::Result<()> {
    loop {
        for _ in 0..BATCH_SIZE {
            let filename = "Canon_40D.jpg";

            let input = std::fs::read(filename).unwrap();
            let mut mutator = lib::Mutator::new(
                input,
                0x2839839283234 ^ unsafe { std::arch::x86_64::_rdtsc() },
            );
            mutator.bitflip(0.01);
            let exit = fuzz(thr_id, filename, &mutator.input).unwrap();
            if exit.signal() == Some(11) {
                // std::fs::write(
                //     format!("./output/crash_{}_{}", stats.num_crashes.load(Ordering::SeqCst), filename),
                //     mutator.input,
                // )
                // .unwrap();
                stats.num_crashes.fetch_add(1, Ordering::SeqCst);
            }
        }
        stats.fuzz_cases.fetch_add(BATCH_SIZE, Ordering::SeqCst);
    }
}

fn main() {
    let mut threads = Vec::new();
    let stat = Arc::new(Statistics::default());
    stat.num_crashes.fetch_add(0, Ordering::SeqCst);

    for thr_id in 0..4 {
        let stat = stat.clone();
        threads.push(std::thread::spawn(move || worker(thr_id, stat)));
    }

    let start = std::time::Instant::now();

    loop {
        std::thread::sleep(Duration::from_millis(1000));
        let elapsed = start.elapsed().as_secs_f64();
        let cases = stat.fuzz_cases.load(Ordering::SeqCst);

        println!(
            "{:10.6} Cases: {:10} | fcps: {:10.2}",
            elapsed,
            cases,
            cases as f64 / elapsed
        );
    }
}
