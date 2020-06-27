//! Reader functionality for sample table and it's descendents.
use crate::mpeg4::boxes::{MP4Box, BOX_HEADER_SIZE, FULL_BOX_HEADER_SIZE};
use crate::mpeg4::formats::{AudioObjectTypes, ChannelConfig};
// use crate::mpeg4::util::dump_buffer;
use bytes::buf::Buf;

// TODO(jdr): This should probably be made into something that can read, video and system
// files, based on the 4 char format dsecription.

// Read Sample Description Box [stsd] assuming audio.
//
// ```spec
// From section R5 8.5.2 Sample Description Box:
//
// aligned(8) class SampleDescriptionBox (unsigned int[32] handler_type)
//     extends FullBox("stsd", version, 0){
//         int i;  // Declares the logical counter below, not in bit stream.
//         unsigned int(32) entry_count;
//         for( i = 1; i <= entry_count; i++) {
//             SampleEntry();  // an instance of a class derived from Sample Entry
//         }
//     }
//

/// Read the basic sound format data out of an MP4A Audio Sample Entry Box
/// channels: number of channels
/// bx: MP4Box fo the MP4A box.
///
/// sapmle_size: bits per sample
///
/// sample_rate: samples per second.
///
/// The generic form of a SampleEntry is:
/// aligned(8) abstract class SampleEntry (unsigned int(32) format)
///     extends Box(format) {
///     const unsigned int(8)[6] resrved = 0;
///     unsigned int(16) data_reference_index;
///     }
///
/// However they're all different. This is specifically for the MP4A variant
/// for audio formating. This is derived and obeys the structure of the
/// QuickTime audio atom.
///  ```
pub fn read_mp4a<'a>(
    bx: &'a mut MP4Box,
    channels: &'a mut u16,
    sample_size: &'a mut u16,
    sample_rate: &'a mut u32,
) {
    bx.buf.advance(BOX_HEADER_SIZE);

    // Next there are 6 bytes reserved as 0.
    bx.buf.advance(6);

    // Data reference_index
    // For MP4A files usually just 1.  Reference to the first and only track?
    let _dref_index = bx.buf.get_u16(); // from dref box.
                                        // println!("Data reference index: {:?}", _dref_index);

    // Specified in the Quicktime File Specificaiton.
    // Version = 0 implies some form of raw or uncompressed
    // Version = 1 implies compressed
    // Version = 2 Introduced in QuitckTime 7 and essentially deprecates Versions 0 and 1.
    //
    //
    // Version 1 implies some form of compression
    // 4 fields of 32 bits each (16 bytes total).
    //    * Samples_per_packet: Number of uncomprssed frames generated by a compressed frame
    //      (compressed frame = 1 sample per channel). Uncompressed formats this is 1.
    //    * Bytes_per_packet: Frame_size / number_of_channels: simpler to sample_size in uncompressed
    //      not that meaningful in compressed.
    //    * Bytes_per_frame: The number of bytes in a frame. (Bytes per packet * number 0f channels).
    //    * Bytes per sample: uncompressed sample size in bytes.
    // If these fields are not used, they are set to 0. So you should only need to check
    // if samplesPerPacket is 0 to know.
    //
    // Version 2 loses the verison 0 and 1 fields
    // The version 2 type BoxType(Atom Format) is "lpcm" (above).
    //
    // version == 2
    // revision == 0
    // vendor == 0
    // always3 == channels == 3
    // always16 == sample_size == 16
    // alwaysMinus3 = audio_compression == -2
    // always0 = packet_size == 0
    // always65536 = sample_rate = 65536        // this is the standard version 0 structure
    // sizeOfStructOnly = u32 providing offset to sound sample structure's extensions.
    // audioSampleRate = f64, 64bit float for sample reate: e.e. 44100.0
    // numAudioChannels = u32 number of channels. channel assignment expressed in an extension
    // always7F000000 = 0x7F_00_00_00 : u32
    // constBitsPerChannel: u32 = set only if constant and only if ucompressed, otherwise 0.
    // formatSpecificFlags: u32 = in LPCM flag Values.
    // constBytesPerAudioPacket: u32 = Number of bytes per packet if constant on if ucompressed, othewrwise 0.
    // constLPCMFramesPerAudioPackate: u32 = set to number of PCM frames per packet only if constant, otherwise 0.
    //
    // LPCM Frame: on uncompressed sample for each channel
    // Packets in version 2: the natural access unit of the format for compressed audio, otherwies an LPCM frame.
    // (See spec for format specific flags).

    let _version = bx.buf.get_u16();
    if _version > 0 {
        eprintln!(
            "MP4A version = {}. Expected 0 likely missing informaiton.",
            _version
        );
    }
    // println!("MP4A ASE version: {:?}", _version as i16);

    // ... also 0 ....
    let _revsion_level = bx.buf.get_u16();
    // println!("ASE revision: {:?}", _revsion_level as i16);

    // ...  also 0.
    let _vendor = bx.buf.get_u32();
    // println!("MP4A ASE vendor: {:?}", box_types::FourCC(_vendor));

    // Now we come to proper audio values.
    *channels = bx.buf.get_u16();
    *sample_size = bx.buf.get_u16();

    // QuickTime says:
    // Set to 0 for version 0 sound descriptions, may be set to -2
    // for some version 1 sound descriptions.
    let _qt_audio_compresion = bx.buf.get_u16();
    // println!("MP4A ASE Compression: {:}", _qt_audio_compresion);

    let _qt_audio_packet_size = bx.buf.get_u16();
    // println!("MP4AASE packet_size: {:}", _qt_audio_packet_size);

    // this is a 16.16 fixed point number
    // 2.5 samples per second would be: 0x0002_0800 (I think)
    // 44,100 sample per seconds is:
    // 0xac44_0000
    // 0xac44 == 44,100
    // 0x0000 == .0
    *sample_rate = bx.buf.get_u32();

    // This only implements the version 0 case at this point.
    // More work to to be done to finish this up.
}

///
/// The MPEG4 Book has a reasonable description of this.
// class ES_Descriptor extends BaseDescriptor : bit
// (8) tag=ES_DescrTag {
//     bit(16) ES_ID;
//     bit(1) streamDependenceFlag;
//     bit(1) URL_Flag;
//     bit(1) OCRstreamFlag;
//     bit(5) streamPriority;
//     if (streamDependenceFlag)
//         bit(16) dependsOn_ES_ID;
//     if (URL_Flag) {
//         bit(8) URLlength;
//         bit(8) URLstring[URLlength];
//     }
//     if (OCRstreamFlag)
//         bit(16) OCR_ES_Id;
//     DecoderConfigDescriptor decConfigDescr;
//     SLConfigDescriptor slConfigDescr;
//     IPI_DescrPointer ipiPtr[0 .. 1];
//     IP_IdentificationDataSet ipIDS[0 .. 255];
//     IPMP_DescriptorPointer ipmpDescrPtr[0 .. 255];
//     LanguageDescriptor langDescr[0 .. 255];
//     QoS_Descriptor qosDescr[0 .. 1];
//     RegistrationDescriptor regDescr[0 .. 1];
//     ExtensionDescriptor extDescr[0 .. 255];
// }
//

// TODO(jdr): Perhaps it's better to return the integers for
// AudioObjectTypes and ChannelConfig and then
// let applications use the Enums as they see fit.
// It's almost certainly trival but it removes the copies and
// the assignment logic.
#[allow(clippy::too_many_arguments)]
pub fn read_esds<'a>(
    bx: &'a mut MP4Box,
    decoder: &'a mut u8,
    avg_bitrate: &'a mut u32,
    max_bitrate: &'a mut u32,
    codec: &'a mut AudioObjectTypes,
    sample_frequency: &mut u32,
    channel_config: &mut ChannelConfig,
    // channels: &'a mut u16,
    // sample_size: &'a mut u16,
    // sample_rate: &'a mut u32,
) {
    bx.buf.advance(FULL_BOX_HEADER_SIZE);

    // println!("read_esds");

    // This may have more than 1 descriptor.
    while bx.buf.len() > 2 {
        // dump_buffer(bx.buf);
        // The tag describes the kind of descriptor this is
        // and therefore how we parse it.
        let tag = bx.buf.get_u8();
        // println!("Read a new ESDS tag: {:0x?}", tag);

        // Length encoding is done from one byte up to four bytes.
        // The first byte that has a 0 in the MSB is the last byte
        // of the total.
        let mut length: u32 = 0;
        for _ in 0..4 {
            // We're done here if we've run out of buffer.
            if bx.buf.is_empty() {
                length = 0;
                break;
            }
            // Get the value
            let l = bx.buf.get_u8();
            // Shift the accumulator left 7 bits.
            // and add the 7 ls bits just read into the accumulator
            length = (length << 7) + ((l & 0x7f) as u32);
            // If the msb of the byte we just read is 0, we're done.
            if (l & 0x80) == 0 {
                break;
            }
        }
        // println!("Descriptor length: {}", length);

        // End should point to the length of the next descriptor.
        const ES_DESCIPTOR: u8 = 3;
        const DECODER_CONFIG_DESCRIPTOR: u8 = 4;
        const DECODER_SPECIFIC_INFO: u8 = 5;
        const TAG6: u8 = 6;

        match tag {
            ES_DESCIPTOR => read_es_descriptor(bx),
            DECODER_CONFIG_DESCRIPTOR => {
                // TODO(jdr): consider just advancing length here.
                read_decoder_config_descriptor(bx, decoder, max_bitrate, avg_bitrate)
            }
            DECODER_SPECIFIC_INFO => {
                read_audio_specific_config(bx, length, codec, sample_frequency, channel_config)
            }
            TAG6 => read_tag6(bx),
            _ => println!("Decoding: BAD TAG!"),
        }
        // println!("Bottom of loop.")
    }
}

/// ES Ddescriptor
///
fn read_es_descriptor(bx: &mut MP4Box) {
    // println!("Decoding: Object Descriptor");

    // ES ID bit(16)
    let _id = bx.buf.get_u16();
    // streamDependenceFlag  bit(1)
    // URL_Flag  bit(1)
    // OCRSstreamFlag bit(1)
    // streamPriority bit(5)
    let flags = bx.buf.get_u8();

    // Stream dependency flag.
    if flags & 0x80 > 0 {
        // let _depends_on_es_id = bx.buf.get_u16();
        bx.buf.advance(2);
    }

    // URL flag
    if flags & 0x40 > 0 {
        // length
        let length = bx.buf.get_u8() as usize;
        // URL String is stored in the length bits following
        bx.buf.advance(length);
    }

    if flags & 0x20 > 0 {
        // let ocr_es_id = bx.buf.get_u16();
        bx.buf.advance(2);
    }
    // println!("ID: {}", id);
    // println!("Flags: {}", flags);
}

/// Get the decoding configuration
// TODO(jdr): Find the *decoder table and turn it into and enumeration.
fn read_decoder_config_descriptor(
    bx: &mut MP4Box,
    decoder: &mut u8,
    max_bitrate: &mut u32,
    avg_bitrate: &mut u32,
) {
    // println!("Decoding elemenary stream descriptor");
    *decoder = bx.buf.get_u8();
    // println!("Type/Profile: {}[{:#04x}]", profile, profile);

    // bit(6) = streamType;
    // bit(1) = upStream;
    // bit(1) = reserved=1;
    // bit(24) = bufferSizeDB;
    let _stream_info = bx.buf.get_u32();
    // println!(
    //     "stream_info - stream_type: {:#04x?}, upstream: {}, reserved: {:0x?}, bufferSize: {:#08x?}",
    //     (0xFC_00_00_00 & _stream_info) >> 26, // streamType
    //     (0x02_00_00_00 & _stream_info) >> 25, // upstream
    //     (0x01_00_00_00 & _stream_info) >> 24, // reserved
    //     0x00_FF_FF_FF & _stream_info          // bufferSize
    // );
    *max_bitrate = bx.buf.get_u32();
    *avg_bitrate = bx.buf.get_u32();

    // bx.buf.advance(12);
    // *decoder = match profile {
    //     0x40 | 0x41 => "AAC".to_string(),
    //     0x68 => ".mp3".to_string(),
    //     _ => format!("Unknown[{:0x?}", profile),
    // };
    // println!("Decoder = {:?}", decoder);
    // println!("maxBitrate: {}", max_bitrate);
    // println!("avg_bitrate: {}", avg_bitrate);
}

/// Sampling Frequencies
/// 0: 96000 Hz
/// 1: 88200 Hz
/// 2: 64000 Hz
/// 3: 48000 Hz
/// 4: 44100 Hz
/// 5: 32000 Hz
/// 6: 24000 Hz
/// 7: 22050 Hz
/// 8: 16000 Hz
/// 9: 12000 Hz
/// 10: 11025 Hz
/// 11: 8000 Hz
/// 12: 7350 Hz
/// 13: Reserved
/// 14: Reserved
/// 15: frequency is written explictly
// TODO(jdr): Consdier only putting 12 elements in this table and
// thereby panicing if we try to access the reserved or extension entries.
const SAMPLE_FREQUENCIES: [u32; 16] = [
    96_000, 88_200, 64_000, 48_000, 44_100, 32000, 24_400, 22_050, 16_000, 12_000, 11_025, 8_000,
    7_350, 0, 0, 0,
];

///
/// This is for the Audio Specific Config.
/// Here is a ref https://wiki.multimedia.cx/index.php/MPEG-4_Audio
/// for layout.
/// This is bitpacked into either 2 bytes, 5 bytes, or 6 bytes.
/// 5 bits: object type
/// 4 bits: frequency index
/// if (frequency index == 15)
///     24 bits: frequency
/// 4 bits: channel configuration
/// 1 bit: frame length flag
/// 1 bit: dependsOnCoreCoder
/// 1 bit: extensionFlag
/// frame length flag:
/// 0: Each packet contains 1024 samples
/// 1: Each packet contains 960 samples.
///
/// length is the length of the decriptor computed in the descriptor header
/// and is in bytes.
/// This is used to speed up the access to the bitpacked value.
fn read_audio_specific_config(
    bx: &mut MP4Box,
    length: u32,
    object_type: &mut AudioObjectTypes,
    frequency: &mut u32,
    channel_config: &mut ChannelConfig,
) {
    // println!("Decoding AUDIO_SPECIFIC_INFO");

    // let object_type: u8;
    // let frequency: u32;
    // let channel_config: u8;
    match length {
        // Simple Structure no extensions - the usual case for most audio files.
        // TODO(Jdr): this isn't quite a DRY as it might be.
        2 => {
            // TODO(jdr): It sure would be good to get rid of these copies into bits and read
            // directly from the buffer.
            let bits = bx.buf.get_u16();
            *object_type = AudioObjectTypes::from(((bits & 0b11111000_00000000) >> 11) as u8);
            let fi = ((bits & 0b00000111_10000000) >> 7) as usize; // It's an index so usize.
            *channel_config = ChannelConfig::from(((bits & 0b00000000_01111000) >> 3) as u8);
            *frequency = SAMPLE_FREQUENCIES[fi];
        }
        // We will assume that the only way this happens is if
        // the object type is an extended object type and the frequency is just the lookup.
        3 => {
            // Read in the buffer for manipulation.
            let mut bits: [u8; 3] = [0; 3];
            for b in &mut bits {
                *b = bx.buf.get_u8();
            }
            // We'll assume that the first 5 bits are 0b11111 == 31. It's the next 6 bits that
            // we care about for object_type
            *object_type = AudioObjectTypes::from(
                32 + ((u16::from_be_bytes([bits[0], bits[1]]) & 0b00000111_11100000) >> 5) as u8,
            );
            // let bits = bx.but.
            let fi = ((bits[1] & 0b000_11110) >> 1) as usize; // It's an index so usize.
            *channel_config = ChannelConfig::from(
                ((u16::from_be_bytes([bits[1], bits[2]]) & 0b00000001_11100000) >> 3) as u8,
            );
            *frequency = SAMPLE_FREQUENCIES[fi];
        }
        // This is should be the case of standard object_type and embedded frequency
        // We'll assume that the frequency index is 0b1111 == 15 and so the
        // frequency will be read from the following 24 bits.
        5 => {
            let mut bits: [u8; 5] = [0; 5];
            for b in &mut bits {
                *b = bx.buf.get_u8();
            }
            *object_type = AudioObjectTypes::from(
                ((u16::from_be_bytes([bits[0], bits[1]]) & 0b11111000_00000000) >> 11) as u8,
            );
            *frequency = ((u32::from_be_bytes([bits[1], bits[2], bits[3], bits[4]])
                & 0b01111111_11111111_11111111_10000000)
                >> 7) as u32;
            *channel_config = ChannelConfig::from(bits[4] & 0b0111_1000);
        }
        // Here we have both extended object type, and embeded frequency (does this ever happen)?
        // We'll assume both extension values are set correctly.
        6 => {
            let mut bits: [u8; 6] = [0; 6];
            for b in &mut bits {
                *b = bx.buf.get_u8();
            }
            // We'll assume that the first 5 bits are 0b11111 == 31. It's the next 6 bits that
            // we care about for object_type
            *object_type = AudioObjectTypes::from(
                32 + ((u16::from_be_bytes([bits[0], bits[1]]) & 0b00000111_11100000) >> 5) as u8,
            );
            // assume that the first 4 bits following the previous are 0b1111 = 15, then
            // read the frequency out of the next 24.
            *frequency = ((u32::from_be_bytes([bits[1], bits[2], bits[3], bits[4]])
                & 0b00000001_11111111_11111111_11111110)
                >> 1) as u32;
            *channel_config = ChannelConfig::from(
                ((u16::from_be_bytes([bits[4], bits[5]]) & 0b00000001_11100000) >> 5) as u8,
            );
        }
        _ => {
            *object_type = AudioObjectTypes::Null;
            *frequency = 0; // this may want to be an option.
            *channel_config = ChannelConfig::Unknown;
            eprintln!("Bad length in esds/audio_specific_descirptor!");
        }
    }
    // println!("Object_type: {}", object_type);
    // println!("frequency: {}", frequency);
    // println!("Channel config: {}", channel_config);
}

/// Don't know what this is for.
fn read_tag6(bx: &mut MP4Box) {
    // println!("Reading tag id: 6");
    let _data = bx.buf.get_u8();
    // println!("Data is; {}", data);
}
