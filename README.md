# CASL
Command-Action Speech Loopback

A fancy way of saying smart speaker.

## Building
Compiling requires an extra step because deepspeech is an external (non-rust) library. 
To install deepspeech, download the latest `native_client` 0.9.X version for your system from [GitHub](https://github.com/mozilla/DeepSpeech/releases/tag/v0.9.3). 

While you're on that page, you should download a model file (both if you're not sure which will work on your platform) and the `deepspeech-0.9.X-models.scorer` file. 
You will need these later when you run CASL. 

Create a `lib` directory and place all of these downloads in there. 

Finally, add that `lib` directory to the `LIBRARY_PATH` and `LD_LIBRARY_PATH` environment variables.

Now that everything is set up, use `cargo` like any other Rust project (eg `cargo run --release` to run an optimised version).

### Notes
The deepspeech Rust wrapper officially supports deepspeech version 0.9.0, so if you have issues with 0.9.3 or later try downgrading the model and lib files to that version. 

Environment variables are not persistent if you just set them with `env`.

The `casl.json` configuration in this repo is configured to use the model and scorer in `./lib/`. 
If you move those, update the config file to reflect those changes.

## Config
The CASL configuration is defined in a file named `casl.json` in CASL's current working directory (e.g. beside the CASL binary). 
CASL will not start without a valid configuration file. 
See `casl.json` in this project for a complete configuration example. 

### Reference

- **model**: Absolute path to the deepspeech model (this should be a `.tflite` or `.pbmm` file depending on your platform).
- **scorer**(optional): Absolute path to the external scorer (omit to use integrated scorer).
- **carryover_buffer_size**: Minimum amount of buffer samples to keep when cleaning up the buffer. 
If speech is detected while a cleanup is attempted, more of the buffer will be kept.
- **refresh_buffer_threshold**: Minimum buffer size to trigger a buffer cleanup. 
The buffer can grow past this size when speech is detected during a buffer cleanup.
- **gap_detection_ms**: Minimum time (milliseconds) to count as a gap between spoken commands.
- **preprocessors**: List of text pre-processor configurations.

## Pre-Processors
Before CASL lets commands handle the converted words, CASL passes the text through a set of text pre-processors. 
These pre-processors are executed in the same order defined in the config file. 

There are two types of pre-processors supported by CASL:
- **Remap**: Maps regex patterns to another string.
- **Redirect**: Wraps another pre-processor so that its configuration can be defined in a separate JSON file.

### API Reference
All configuration files are JSON dictionary objects with string keys and any type of value. 
There is no limit to the amount of pre-processors you can define, nor is there a limit on the amount of a single type of pre-processor. 

#### Remap
The Remap pre-processor requires two key-value pairs; one to indicate it's a Remap pre-processor and one to define the remappings.
- **type**: The pre-processor type name. For Remap pre-processors, this should always be `"Remap"`.
- **mappings**: Dictionary of the format `"search regex": "replacement text"` to specify words and phrases to replace with another word or phrase. 
The replacement text may use `$` to reference capture groups as defined in the [Rust Regex docs](https://docs.rs/regex/1.4.3/regex/struct.Regex.html#replacement-string-syntax).

**NOTE**: Regular Expressions use a lot of backslashes \\ characters. 
These must be escaped by another backslash \\\\ for it to be a valid JSON and interpreted properly by Rust's regex compiler.

The first mapping in this example replaces any mention of a blue screen with a kernel panic (because Linux is better), 
while the second mapping replaces any mention of ingenious with my username (because CASL can't understand my username properly). 
Both mappings preserve the space or end of text after them to reduce interference with other pre-processors and other CASL functionality.
```JSON
{
  "type": "Remap",
  "mappings": {
    "blue screen(s?)(\\\\s|$)": "kernel panic$1$2",
    "((ingenious)|(in genie us))(\\\\s|$)" : "ngnius$4"
  }
}
```

#### Redirect
The Redirect pre-processor requires two key-value pairs; one to indicate it's a Redirect type and another to define the new config JSON's filepath.
- **type**: The pre-processor type name. For Redirect pre-processors, this should always be `"Redirect"`. 
- **path**: The path to the JSON config file to use for the underlying pre-processor. This config file should be a valid pre-processor JSON object. 

**NOTE**: Windows-style filepathes have a lot of backslashes \\ in them. 
These must be escaped by adding another backslash \\\\ for it to be a valid JSON and filepath. 
Alternately use Unix-style filepathes with forward slashes / instead.

For example, this configuration tells CASL to use the pre-processor config file located at `/home/ngnius/remap_nato.json`. 
```JSON
{
  "type": "Redirect",
  "path": "/home/ngnius/remap_nato.json"
}
```
Presumably, `/home/ngnius/remap_nato.json` contains the NATO phonetic alphabet for spelling out words which CASL can't decode, 
as does the `casl.json` file in this project. 

## Commands
CASL revolves around commands and actions. 
Sound is captured from the microphone and automatically converted into text using deepspeech and some text preprocessors. 
The string of text is then sent to command processors to determine what action needs to be done. 

There are two ways to communicate with CASL from a command processor:
- **Net**: Use UDP socket networking to send and receive information
- **StdIO**: Use stdin to receive information, stdout to send information, and stderr for debugging

### API Reference
// TODO
(see examples and casl.json in the meantime)
