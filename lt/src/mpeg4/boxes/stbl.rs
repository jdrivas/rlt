use crate::mpeg4::boxes::box_types::{BoxType, ContainerType};
use crate::mpeg4::boxes::MP4Box;
use crate::mpeg4::util;
use bytes::buf::Buf;
use util::dump_buffer;

// TODO(jdr): This should probably be made into something that can read, video and system
// files, based on the 4 char format dsecription.

/// Retrieve sample description data from an 'stsd' box directly into arguments.
///
/// format: 4 char format descriptor usually b"mp4a", but could be eb"mca", b"samr", b"sawb", and probably others.
///
/// channels: number of channels
///
/// sapmle_size: bits per sample
///
/// sample_rate: samples per second.
///
pub fn get_short_audio_stsd<'a>(
    bx: &'a mut MP4Box,
    format: &'a mut [u8; 4],
    channels: &'a mut u16,
    sample_size: &'a mut u16,
    sample_rate: &'a mut u32,
) {
    // get_audio_stsd(bx, format, channels, sample_size, sample_rate, &mut None);
    get_audio_stsd(bx, format, channels, sample_size, sample_rate);
}

/// Read Sample Description Box [stsd]
///
/// ```?:
/// From section R5 8.5.2 Sample Description Box:
///
/// aligned(8) class SampleDescriptionBox (unsigned int[32] handler_type)
///     extends FullBox("stsd", version, 0){
///         int i;  // Declares the logical counter below, not in bit stream.
///         unsigned int(32) entry_count;
///         for( i = 1; i <= entry_count; i++) {
///             SampleEntry();  // an instance of a class derived from Sample Entry
///         }
///     }
///
/// aligned(8) abstract class SampleEntry (unsigned int(32) format)
///     extends Box(format) {
///     const unsigned int(8)[6] resrved = 0;
///     unsigned int(16) data_reference_index;
///     }
///
///  ```
/// Also from the spec:
///
/// > Version is set to 0, unless the box contains an AudioSampleEntryV1
/// whereupon it must be 1
///
/// > entry_count is an integer that gives the number of entries in the following table
///
/// > Sample_Entry is the appropriate sample entry
///
/// > data_reference_index is an integer that contains the index of the data reference
/// to use to retrieve data assoicated with samples that use this sample description.
///
/// > Data references are stored in Data Reference Boxes. This index ranges from 1 to the number
/// of data references.
pub fn get_audio_stsd<'a>(
    bx: &'a mut MP4Box,
    _format: &'a mut [u8; 4],
    channels: &'a mut u16,
    sample_size: &'a mut u16,
    sample_rate: &'a mut u32,
) {
    println!("STSD Box after header read.");
    dump_buffer(bx.buf);
    let _entry_count = bx.buf.get_u32(); // should equal 1 for the audio files we're looking at.

    //
    // Sample Entry Box
    println!("Sample Entry Box:");
    dump_buffer(bx.buf);

    // Sample Entry Box Size
    let _len_desc = bx.buf.get_u32();

    // Sample Entry Box type
    // We expect b"mp4a".
    //      Rumour has it that we could get: b"emca", b"samr", b"sawb";
    let _se_type = bx.buf.get_u32();

    // Next there are 6 bytes rserved as 0.
    bx.buf.advance(6);

    // Data reference_index
    let _dref_index = bx.buf.get_u16(); // from dref box.

    // Old style QT .mov format
    let _qt_enc_version = bx.buf.get_u16(); // quicktime audio encoding version
                                            // if *qt_enc_ver != None {
                                            //     *qt_enc_ver = Some(bx.buf.get_u16());
                                            // } else {
                                            //     bx.buf.advance(2);
                                            // }
    let _qt_audio_rev = bx.buf.get_u16(); // quicktime audio encoding revision.
    let _qt_vendor = bx.buf.get_u32(); // quicktime audio encoding vendor, 4 byte ascii string: b"XXXX".

    // Proper audio values
    *channels = bx.buf.get_u16();
    *sample_size = bx.buf.get_u16();

    // More QT MOV format information
    let _qt_audio_compresion = bx.buf.get_u16(); // defined as 0 here.
    let _qt_audio_packet_size = bx.buf.get_u16(); // defined as 0 here.

    // this is a 16.16 fixed point number
    *sample_rate = bx.buf.get_u32();

    // This is the ESDS box?
    println!("ESDS box:");
    dump_buffer(bx.buf);

    // match qt_enc_version {
    //     0 => (),

    //     // Quicktime sound sample description version 1.
    //     1 => bx.buf.advance(16), // move past unknown QT fields.

    //     // Quicktime sound sample description version 1.
    //     2 => {
    //         bx.buf.advance(4); // move past unknown QT fields
    //         let sr = f64::from_bits(bx.buf.get_u64()); // TODO(jdr): Rationalize what we return.
    //         let cc = bx.buf.get_u64(); // TODO(jdr): Decide how to convert.
    //         bx.buf.advance(20);
    //     }
    //     _ => (), // TODO(jdr): Need to error out.
    // }
    /*
    // check to see if the BOXEs above
    // are acceptable?
    match  se_type {
        b".mp3" |      // MP3 Audio Sample Type
        b"lpcm" => (), // LPCM (defined in quicktime for, presumably, linear pulse code moduleation) Type
        MP4A => (),   // "Expected" MP4 Audio files.
        _ => (),       // TODO(jdr): need to error out.
    }

    match b"abcd" {
        b"dfLa" => (), // FLAC SampleEntryBox
        b"dOps" => (), // Opus SampleEntryBox
        b"alac" => (), // ALAC Type

        &ESDS => (),
        _ => (), // TODO(jdr): need to error out.
    }
    */
}
