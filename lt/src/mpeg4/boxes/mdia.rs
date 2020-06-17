//! Reader functionality for Media Data and it's header.

use crate::mpeg4::boxes::{MP4Box, FULL_BOX_HEADER_SIZE};
use bytes::buf::Buf;

/// Media Header Box
/// creation and modification times are seconds since midnight 1/1/04 in UTC.
/// creation: Creation fo the track.
/// modification: Last modification of the track.
/// timescale: number of units that pass in a second.
/// duration: in units of the timescale (e.g. samples);
/// language: ISO 639-2/T Representes as 16 bits be: 0111112222233333.
/// where each digit is a bit of a five bit ascii char code. There are 3
/// of these - 3 lower case ascii characters
pub fn get_mdhd<'a>(
    bx: &'a mut MP4Box,
    creation: &'a mut u64,
    modification: &'a mut u64,
    timescale: &'a mut u32,
    duration: &'a mut u64,
    language: &'a mut u16,
) {
    // Move past the fullbox header.
    bx.buf.advance(FULL_BOX_HEADER_SIZE);
    if let Some(vf) = &bx.version_flag {
        if vf.version == 1 {
            *creation = bx.buf.get_u64();
            *modification = bx.buf.get_u64();
            *timescale = bx.buf.get_u32();
            *duration = bx.buf.get_u64();
        } else {
            *creation = bx.buf.get_u32() as u64;
            *modification = bx.buf.get_u32() as u64;
            *timescale = bx.buf.get_u32();
            *duration = bx.buf.get_u32() as u64;
        }
        *language = bx.buf.get_u16();
    } else {
        panic!("mdhd didn't read a BoxType::Full so had no version flag.");
    }
}
