use std::sync::mpsc::{Receiver, Sender};
use deepspeech::{Model, Stream};

use crate::config;

pub fn process_audio_loop(cntrl: Receiver<bool>, audio: Receiver<i16>, casl_config: &config::Config) {
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

pub fn process_audio(sample_rx: &Receiver<i16>, stream: &mut Stream, buffer: &mut Vec<i16>, max: usize) {
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

pub fn process_metadata(metadata: &deepspeech::Metadata) {
    let mut text = String::new();
    for token in metadata.transcripts()[0].tokens() {
        text += token.text().unwrap();
    }
    println!("Heard: {}", text);
}

pub fn capture_audio(data: &[f32], _: &cpal::InputCallbackInfo, audio_tx: Sender<i16>) {
    for &sample in data {
        let int_sample: i16 = (sample*((std::i16::MAX) as f32)) as i16;
        //println!("{} f32 -> {} i16", sample, int_sample);
        audio_tx.send(int_sample).unwrap_or(()); // ignore errors
    }
}

pub fn capture_error(error: cpal::StreamError) { println!("Stream error {:?}", error); }