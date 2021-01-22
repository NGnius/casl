use std::sync::mpsc::{Receiver, Sender};
use deepspeech::{Model, Stream};

use crate::{config, TARGET_SAMPLE_RATE};

const TIMESTEP_TO_MS: u32 = 20; // 20ms increments

pub fn process_audio_loop(cntrl: Receiver<bool>, audio: Receiver<i16>, casl_config: &config::Config) {
    // init
    let mut speech2text = Model::load_from_files(std::path::Path::new(&casl_config.model)).unwrap();
    if let Some(scorer) = &casl_config.scorer {
        speech2text.enable_external_scorer(std::path::Path::new(scorer)).unwrap();
    }
    let mut stream = speech2text.create_stream().unwrap();
    let mut buffer = std::vec::Vec::with_capacity(casl_config.refresh_buffer_threshold);
    let mut is_exiting = cntrl.try_recv().unwrap_or(false);
    let mut last_carryover = 0;
    while !is_exiting {
        // process audio until exit signal is received
        process_audio(&audio, &mut stream, &mut buffer, casl_config.carryover_buffer_size);
        if buffer.len() - last_carryover >= casl_config.refresh_buffer_threshold {
            // decode audio
            let text = stream.finish_with_metadata(1).unwrap();
            let meta = process_metadata(&text,
                                        (buffer.len() as u32 / (TARGET_SAMPLE_RATE / 1_000)) as u32,
                                        casl_config);
            // refresh stream
            stream = speech2text.create_stream().unwrap();
            if meta.safe_to_refresh {
                crate::command::process_commands(&meta, casl_config);
                //println!("Handling: `{}` -> `{}`", meta.phrase_raw, meta.phrase);
                let mut carryover = std::vec::Vec::with_capacity(casl_config.carryover_buffer_size);
                carryover.extend(&buffer[buffer.len()-casl_config.carryover_buffer_size..]);
                buffer = std::vec::Vec::with_capacity(casl_config.refresh_buffer_threshold);
                buffer.extend(&carryover);
            } else {
                let gap_start_sample = (meta.last_gap_end_ms * TARGET_SAMPLE_RATE / 1000) as usize;
                let mut carryover = std::vec::Vec::with_capacity(buffer.len()-gap_start_sample);
                carryover.extend(&buffer[buffer.len()-gap_start_sample..]);
                buffer = std::vec::Vec::with_capacity(casl_config.refresh_buffer_threshold);
                buffer.extend(&carryover);
            }
            last_carryover = buffer.len();
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

pub fn process_metadata(metadata: &deepspeech::Metadata, length_ms: u32, casl_config: &config::Config) -> MetadataResult {
    let mut text = String::new();
    let transcript = &metadata.transcripts()[0];
    // TODO detect gaps to use as start of sentence
    let mut last_sound: u32 = 0;
    let mut last_gap: u32 = 0;
    for token in transcript.tokens() {
        if (token.timestep() - last_sound) * TIMESTEP_TO_MS > casl_config.gap_detection_ms as u32 {
            // clear text if gap detected (only keep data after end of last recent gap)
            text.clear();
            last_gap = token.timestep();
        }
        text += token.text().unwrap();
        if token.timestep() > last_sound && !token.text().unwrap().chars().all(char::is_whitespace) {
            // keep track of when the last valid token was sent (the start of the gap)
            last_sound = token.timestep();
        }
    }
    let ends_with_gap: bool = length_ms - (last_sound * TIMESTEP_TO_MS) > casl_config.gap_detection_ms as u32;
    // preprocess text
    let mut processed_text = text.clone();
    for pre in &casl_config.preprocessors {
        processed_text = pre.preprocessor().process(&processed_text).to_string();
    }
    if last_gap != 0 { last_gap -= 1; } // buffer zone, deepspeech is only accurate to ~20ms
    MetadataResult {
        safe_to_refresh: ends_with_gap,
        phrase_raw: text,
        phrase: processed_text,
        last_gap_start_ms: (last_sound+1) * TIMESTEP_TO_MS,
        last_gap_end_ms: last_gap * TIMESTEP_TO_MS,
    }
}

pub fn capture_audio(data: &[f32], _: &cpal::InputCallbackInfo, audio_tx: Sender<i16>) {
    for &sample in data {
        let int_sample: i16 = (sample*((std::i16::MAX) as f32)) as i16;
        //println!("{} f32 -> {} i16", sample, int_sample);
        audio_tx.send(int_sample).unwrap_or(()); // ignore errors
    }
}

pub fn capture_error(error: cpal::StreamError) { println!("Stream error {:?}", error); }

#[derive(Clone)]
pub struct MetadataResult {
    pub safe_to_refresh: bool,
    pub phrase_raw: String,
    pub phrase: String,
    pub last_gap_start_ms: u32,
    pub last_gap_end_ms: u32
}