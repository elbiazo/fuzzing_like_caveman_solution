use libafl::bolts::current_nanos;
use libafl::bolts::rands::StdRand;
use libafl::bolts::shmem::{ShMem, ShMemProvider, StdShMemProvider};
use libafl::bolts::tuples::tuple_list;
use libafl::corpus::{
    Corpus, InMemoryCorpus, IndexesLenTimeMinimizerCorpusScheduler, OnDiskCorpus,
    QueueCorpusScheduler,
};
use libafl::events::SimpleEventManager;
use libafl::executors::{ForkserverExecutor, TimeoutForkserverExecutor};
use libafl::feedbacks::{MapFeedbackState, MaxMapFeedback, TimeFeedback};
use libafl::inputs::BytesInput;
use libafl::mutators::{havoc_mutations, StdScheduledMutator};
use libafl::observers::{ConstMapObserver, HitcountsMapObserver, TimeObserver};
use libafl::stages::StdMutationalStage;
use libafl::state::{HasCorpus, StdState};
use libafl::stats::SimpleStats;
use libafl::{feedback_and_fast, feedback_or, Fuzzer, StdFuzzer};
use std::path::PathBuf;
use std::time::Duration;
use libafl::feedbacks::CrashFeedback;

const MAP_SIZE: usize = 65536;
fn main() {
    let corpus_dirs = vec![PathBuf::from("./corpus")];
    let input_corpus = InMemoryCorpus::<BytesInput>::new();

    let crash_corpus = OnDiskCorpus::new(PathBuf::from("./crashes"))
        .expect("Could not create timeouts corpus");



    let time_observer = TimeObserver::new("time");

    let mut shmem = StdShMemProvider::new().unwrap().new_map(MAP_SIZE).unwrap();
    shmem
        .write_to_env("__AFL_SHM_ID")
        .expect("Could not write to env");
    
    let mut shmem_map = shmem.map_mut();

    let edges_observer = HitcountsMapObserver::new(ConstMapObserver::<_, MAP_SIZE>::new(
        "edges",
        &mut shmem_map,
    ));

    let feedback_state = MapFeedbackState::with_observer(&edges_observer);
    
    let feedback = feedback_or!(
        MaxMapFeedback::new_tracking(&feedback_state, &edges_observer, true, false),
        TimeFeedback::new_with_observer(&time_observer)
    );
    let objective_state = MapFeedbackState::new("timeout_edges", MAP_SIZE);


    let objective = feedback_and_fast!( MaxMapFeedback::new(&objective_state, & edges_observer), CrashFeedback::new());

    let mut state = StdState::new(
        StdRand::with_seed(current_nanos()),
        input_corpus,
        crash_corpus,
        tuple_list!(feedback_state, objective_state),
    );

    let stats = SimpleStats::new(|s| println!("{}", s));

    let mut mgr = SimpleEventManager::new(stats);

    let scheduler = IndexesLenTimeMinimizerCorpusScheduler::new(QueueCorpusScheduler::new());

    let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);
    let fork_server = ForkserverExecutor::new(
        "./exif".to_string(),
        &[String::from("@@")],
        false,  // use_shmem_testcase
        tuple_list!(edges_observer, time_observer),
    ).unwrap();

    let timeout = Duration::from_millis(5000);

    // ./pdftotext @@
    let mut executor = TimeoutForkserverExecutor::new(fork_server, timeout).unwrap();


    if state.corpus().count() < 1 {
        state
            .load_initial_inputs(&mut fuzzer, &mut executor, &mut mgr, &corpus_dirs)
            .unwrap_or_else(|err| {
                panic!(
                    "Failed to load initial corpus at {:?}: {:?}",
                    &corpus_dirs, err
                )
            });
        println!("We imported {} inputs from disk.", state.corpus().count());
    }

    let mutator = StdScheduledMutator::new(havoc_mutations());
    let mut stages = tuple_list!(StdMutationalStage::new(mutator));

    fuzzer
    .fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)
    .expect("Error in the fuzzing loop");
}
