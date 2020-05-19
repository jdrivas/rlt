use crate::mpeg4::boxes::MP4Box;
use bytes::buf::Buf;

pub const STBL: [u8; 4] = *b"stbl"; // Sample Table Box Container     /moov/trak/mdia/minf/stbl
pub const STCO: [u8; 4] = *b"stco"; // Chunk Offsets
pub const STSC: [u8; 4] = *b"stsc"; // Sample to Chumk
pub const STSD: [u8; 4] = *b"stsd"; // Sample Description (code types, initialization).
pub const STTS: [u8; 4] = *b"stts"; // Time to sample.
pub const STSZ: [u8; 4] = *b"stsz"; // Sample Sizes

// TODO(jdr): This should probably be made into something that can read, video and system
// files, based on the 4 char format dsecription.

/// Retrieve sample description data from an 'stsd' box directly into the provider
/// &muts
/// format: 4 char format descriptor usually b"mp4a", but could be eb"mca", b"samr", b"sawb", and probably others.
/// channels: number of channels
/// sapmle_size: bits per sample
/// sample_rate: samples per second.
pub fn get_short_audio_stsd<'a>(
    bx: &'a mut MP4Box,
    format: &'a mut [u8; 4],
    channels: &'a mut u16,
    sample_size: &'a mut u16,
    sample_rate: &'a mut u32,
) {
    get_audio_stsd(bx, format, channels, sample_size, sample_rate, &mut None);
}

pub fn get_audio_stsd<'a>(
    bx: &'a mut MP4Box,
    format: &'a mut [u8; 4],
    channels: &'a mut u16,
    sample_size: &'a mut u16,
    sample_rate: &'a mut u32,
    qt_enc_ver: &mut Option<u16>,
) {
    // Read in an audio description.
    let _total_desc = bx.buf.get_u32(); // should equal 1
    let _len_desc = bx.buf.get_u32();

    // Format
    // format = bx.buf.get_u32(); // b"mp4a", b"emca", b"samr", b"sawb"; If not one of these then not audio?
    format.copy_from_slice(&bx.buf[0..4]);
    bx.buf.advance(4 + 6); // Advance the 4 bytes from the format copy, and past the six bytes of reserved set to 0.

    let _dref_index = bx.buf.get_u16(); // from dref box.

    // Quicktime specific.
    // let qt_audio_enc_ver = bx.buf.get_u16(); // quicktime audio encoding version
    if *qt_enc_ver != None {
        *qt_enc_ver = Some(bx.buf.get_u16());
    } else {
        bx.buf.advance(2);
    }

    let _qt_audio_rev = bx.buf.get_u16(); // quicktime audio encoding revision.
    let _qt_vendor = bx.buf.get_u32(); // quicktime audio encoding vendor, 4 byte ascii string: b"XXXX".

    // Proper audio values
    *channels = bx.buf.get_u16();
    *sample_size = bx.buf.get_u16();

    let _qt_audio_compresion = bx.buf.get_u16(); // defined as 0 here.

    // TODO(jdr):
    // Somewhere my specification is bad.
    //The channels and sample size box are correct
    // But only by deleting this next one (or of course the previous one),
    // Will I get the correct sample_rate.
    // let _qt_audio_packet_size = bx.buf.get_u16();  // defined as 0 here.

    *sample_rate = bx.buf.get_u32();
}
