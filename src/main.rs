// CASL (Command-Action Speech Loopback)
mod command;
mod config;

use deepspeech::{Model, Stream};

use std::sync::mpsc::{channel, Receiver, Sender};
use std::vec::Vec;
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
    let audio_thread = std::thread::spawn(move || {
        process_audio_loop(audio_thread_cntrl_rx, audio_thread_sample_rx, &casl_config);
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
        capture_audio(a, b, audio_thread_sample_tx.clone());
    }, capture_error).unwrap();

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

fn process_audio_loop(cntrl: Receiver<bool>, audio: Receiver<i16>, casl_config: &config::Config) {
    // init
    let mut speech2text = Model::load_from_files(std::path::Path::new(&casl_config.model)).unwrap();
    if let Some(scorer) = &casl_config.scorer {
        speech2text.enable_external_scorer(std::path::Path::new(scorer)).unwrap();
    }
    let mut stream = speech2text.create_stream().unwrap();
    let mut buffer = std::vec::Vec::with_capacity(casl_config.refresh_buffer_threshold);
    let mut is_exiting = cntrl.try_recv().unwrap_or(false);
    while !is_exiting {
        // process audio until exit signal is received
        process_audio(&audio, &mut stream, &mut buffer, casl_config.carryover_buffer_size);
        if buffer.len() >= casl_config.refresh_buffer_threshold {
            // decode audio
            let text = stream.finish_with_metadata(1).unwrap();
            process_metadata(&text);
            // refresh stream
            stream = speech2text.create_stream().unwrap();
            let mut carryover = std::vec::Vec::with_capacity(casl_config.carryover_buffer_size);
            carryover.extend(&buffer[buffer.len()-casl_config.carryover_buffer_size..]);
            buffer = std::vec::Vec::with_capacity(casl_config.refresh_buffer_threshold);
            buffer.extend(&carryover);
            drop(carryover);
            stream.feed_audio(&buffer);
        }
        is_exiting = cntrl.try_recv().unwrap_or(false);
    }
}

fn process_audio(sample_rx: &Receiver<i16>, stream: &mut Stream, buffer: &mut Vec<i16>, max: usize) {
    let mut count = 0;
    let mut sample = sample_rx.recv();
    let start = buffer.len();
    while sample.is_ok() && count < max {
        //println!("Sample: {}", sample.unwrap());
        buffer.push(sample.unwrap());
        sample = sample_rx.recv();
        count+=1;
    }
    stream.feed_audio(&buffer[start..]);
}

fn process_metadata(metadata: &deepspeech::Metadata) {
    let mut text = String::new();
    for token in metadata.transcripts()[0].tokens() {
        text += token.text().unwrap();
    }
    println!("Heard: {}", text);
}

fn capture_audio(data: &[f32], _: &cpal::InputCallbackInfo, audio_tx: Sender<i16>) {
    for &sample in data {
        let int_sample: i16 = (sample*((std::i16::MAX) as f32)) as i16;
        //println!("{} f32 -> {} i16", sample, int_sample);
        audio_tx.send(int_sample).unwrap_or(()); // ignore errors
    }
}

fn capture_error(error: cpal::StreamError) { println!("Stream error {:?}", error); }
