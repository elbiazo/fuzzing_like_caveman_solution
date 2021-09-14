use std::io;
// use std::fs;
use std::process::{Command, ExitStatus};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

const BATCH_SIZE: usize = 100;

#[derive(Default)]
struct Statistics {
    fuzz_cases: AtomicUsize 
}

fn fuzz(thr_id: usize, filename: &str, inp: &[u8]) -> io::Result<ExitStatus>{
    // Write out the input to a temporary file
    std::fs::write(format!("./output/tmp_{}_{}", thr_id, &filename[..filename.len()-4]), inp).unwrap();

    let runner = Command::new("exif")
        .arg(filename)
        .output()?;
    
    Ok(runner.status)

}

fn worker(thr_id: usize, stats: Arc<Statistics>) -> io::Result<()>
{
    loop {
        for _ in 0..BATCH_SIZE{
            let filename = "Canon_40D.jpg";
            fuzz(thr_id, filename, b"hello world")?;
        }
    
        stats.fuzz_cases.fetch_add(BATCH_SIZE, Ordering::SeqCst);
    }
}

fn main() {
    let mut threads = Vec::new();
    let stat = Arc::new(Statistics::default());

    for thr_id in 0..4 {
        let stat = stat.clone();
        threads.push(std::thread::spawn(move || worker(thr_id, stat)));
    }

    let start =std::time::Instant::now();

    loop{
        std::thread::sleep(Duration::from_millis(1000));
        let elapsed = start.elapsed().as_secs_f64();
        let cases = stat.fuzz_cases.load(Ordering::SeqCst); 

        println!("{:10.6} Cases: {:10} | fcps: {:10.2}", elapsed, cases, cases as f64 /elapsed);
    }
}
