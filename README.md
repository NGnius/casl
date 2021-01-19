# CASL
Command-Action Speech Loopback

A fancy way of saying smart speaker.

## Config
The CASL configuration is defined in a file named `casl.json` in CASL's current working directory (e.g. beside the CASL binary). 
CASL will not start without a valid configuration file.
See `casl.json` in this project for a sample configuration. 

### Reference

- **model**: Absolute path to the deepspeech model (this should be a `.tflite` or `.pbmm` file depending on your platform)
- **scorer**(optional): Absolute path to the external scorer (omit to use integrated scorer)
- **carryover_buffer_size**: Amount of buffer samples to keep when cleaning up the buffer
- **refresh_buffer_threshold**: Minimum buffer size to trigger a buffer cleanup


## Commands
CASL revolves around commands and actions. 
Sound is captured from the microphone and automatically converted into text using deepspeech and some text preprocessors. 
The string of text is then sent to command processors to determine what action needs to be done. 

There are two ways to communicate with CASL from a command processor:
- **Net**: Use UDP socket networking to send and receive information
- **StdIO**: Use stdin to receive information, stdout to send information, and stderr for debugging

### API Reference
