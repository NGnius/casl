// CASL (Command-Action Speech Loopback)
mod command;
mod config;
mod speech;
mod preprocessor;

use std::sync::mpsc::{channel};
use cpal::traits::{DeviceTrait, HostTrait};

const TARGET_SAMPLE_RATE: u32 = 16_000;

fn main() -> Result<(), ()> {
    println!("CASL, hello!");

    // init
    let json_file = std::fs::File::open("casl.json").unwrap();
    let json_reader = std::io::BufReader::new(json_file);
    let casl_config: config::Config = serde_json::from_reader(json_reader).unwrap();

    // start audio processing thread
    let (audio_thread_cntrl_tx, audio_thread_cntrl_rx) = channel();
    let (audio_thread_sample_tx, audio_thread_sample_rx) = channel();
    let audio_conf = casl_config.clone();
    let audio_thread = std::thread::spawn(move || {
        speech::process_audio_loop(audio_thread_cntrl_rx, audio_thread_sample_rx, &audio_conf);
    });

    // start audio capturing thread
    let host = cpal::default_host();
    let input_device = host.default_input_device().expect("No input device found");
    let mut config: Option<cpal::StreamConfig> = None;
    for sconf in input_device.supported_input_configs().unwrap() {
        if sconf.min_sample_rate().0 <= TARGET_SAMPLE_RATE
            && sconf.max_sample_rate().0 >= TARGET_SAMPLE_RATE {
            let built_config = sconf.with_sample_rate(cpal::SampleRate(TARGET_SAMPLE_RATE)).config();
            //println!("Using config with sample rate {:?} and {} channels", built_config.sample_rate, built_config.channels as u16);
            config = Some(built_config);
            break;
        }
    }
    let input_stream = input_device.build_input_stream(&config.expect("Input device does not support 16kHz mode"), move |a, b| {
        speech::capture_audio(a, b, audio_thread_sample_tx.clone());
    }, speech::capture_error).unwrap();

    // ready (debug info)
    println!("Model {}", &casl_config.model);
    println!("Scorer {}", &casl_config.scorer.unwrap_or("[internal]".to_owned()));
    println!("CASL, ready! ({} pre-processors, {} commands)", casl_config.preprocessors.len(), 0);

    // wait for interrupt signal
    let (stop_tx, stop_rx) = channel();
    ctrlc::set_handler( move || {
        audio_thread_cntrl_tx.send(true).unwrap();
        stop_tx.send(true).unwrap();
    }).unwrap();
    let mut is_exiting = stop_rx.recv();
    while is_exiting.is_ok() && !is_exiting.unwrap() {
        is_exiting = stop_rx.recv();
    }

    // cleanup
    println!("CASL, goodbye!");
    drop(input_stream);
    audio_thread.join().unwrap();
    Ok(())
}
