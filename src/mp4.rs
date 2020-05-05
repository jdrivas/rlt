extern crate bytes;
extern crate lazy_static;
// use lazy_static;
use crate::file;
use crate::file::FileFormat;
use crate::track;
// use byteorder::{BigEndian, ReadBytesExt};
use bytes::buf::Buf;
// use mp4parse;
// use std::convert::TryFrom;
// use std::collections::HashSet;
use std::error::Error;
use std::io::{Read, Seek};
use std::str::from_utf8;
// use std::string::String;

pub struct Mp4;

const FTYP_HEADER: &[u8] = b"ftyp";
const M42_HEADER: &[u8] = b"mp42";
const M4A_HEADER: &[u8] = b"M4A ";
// const M4B_HEADER: &[u8] = b"M4B ";
// const M4P_HEADER: &[u8] = b"M4P ";

pub fn identify(b: &[u8]) -> Option<FileFormat> {
    let mut ft = None;
    if b.len() >= 12 {
        if &b[4..8] == FTYP_HEADER {
            ft = match &b[8..12] {
                b if b == M42_HEADER => Some(FileFormat::MP4A(Mp4 {})),
                b if b == M4A_HEADER => Some(FileFormat::MP4A(Mp4 {})),
                // b if b == M4B_HEADER => return Some(FileFormat::MP4B),
                // b if b == M4P_HEADER => return Some(FileFormat::MP4P),
                _ => None,
            };
        }
    }

    return ft;
}

const FORMAT_NAME: &str = "mpeg-4";
impl file::Decoder for Mp4 {
    fn name(&self) -> &str {
        FORMAT_NAME
    }

    fn get_track(
        &mut self,
        mut r: impl Read + Seek,
    ) -> Result<Option<track::Track>, Box<dyn Error>> {
        // Starthere.
        let mut vbuf = Vec::<u8>::new();
        let _n = r.read_to_end(&mut vbuf)?;
        let mut buf = vbuf.as_slice();
        read_file(&mut buf)?;
        // read_boxes(&mut buf)?;

        return Ok(None);

        /*
                let mut mc = mp4parse::MediaContext::new();
                match mp4parse::read_mp4(&mut r, &mut mc) {
                    Ok(_) => (),
                    Err(e) => eprintln!("Failed to parse mp4:{:?}", e),
                }

                // println!("timescale: {:?}", mc.timescale);
                // println!("Mnovied extends Box: {:?}", mc.mvex);
                // println!("Number of ProtectionSystemSpecificHeaders: {:?}", mc.psshs);
                // println!("Tracks: {}", mc.tracks.len());
                for t in &mc.tracks {
                    //     println!("ID: {}", t.id);
                    //     println!("Type: {:?}", t.track_type);
                    //     println!("Empty Duration: {:?}", t.empty_duration);
                    //     println!("Media Time: {:?}", t.media_time);
                    // println!("Timescale: {:?}", t.timescale);
                    // println!("Duration: {:?}", t.duration);
                    //     println!("Track ID: {:?}", t.track_id);
                    //     println!("Track Header: {:?}", t.tkhd);
                    //     match &t.stsd {
                    //         Some(sd) => {
                    //             println!("Sample Description boxes: {:?}", sd.descriptions.len());
                    //             println!("Sample Entry: {:?}", sd.descriptions[0]);
                    //         }
                    //         None => println!("Sample Description boxes: None"),
                    //     }
                }

                // TODO(jdr): Fix this and decide how to handle multiple tracks.
                // Assume 1 track
                let mut tk = track::Track {
                    ..Default::default()
                };

                if mc.tracks.len() > 0 {
                    if mc.tracks.len() != 1 {
                        eprintln!("There were {} tracks, excepted 1.", mc.tracks.len());
                    }
                    if let Some(sd) = &mc.tracks[0].stsd {
                        if sd.descriptions.len() > 0 {
                            if sd.descriptions.len() != 1 {
                                eprintln!(
                                    "There were {} sample description entries, expected 1",
                                    sd.descriptions.len()
                                );
                            }
                            match &sd.descriptions[0] {
                                mp4parse::SampleEntry::Audio(ase) => {
                                    let mut duration = 0;
                                    if let Some(d) = mc.tracks[0].duration {
                                        duration = d.0;
                                    }
                                    tk.format = Some(track::CodecFormat::PCM(track::PCMFormat {
                                        sample_rate: ase.samplerate as u32,
                                        bits_per_sample: ase.samplesize,
                                        total_samples: duration,
                                        ..Default::default()
                                    }));
                                    // eprintln!("Codec Specific: {:?}", ase.codec_specific);
                                    // eprintln!(
                                    //     "Protection Info size: {} entires",
                                    //     ase.protection_info.len()
                                    // );
                                }
                                _ => eprintln!("Non-audio format."),
                            }
                        }
                    }
                }

                // let _ = read_atoms(r)?;
                return Ok(Some(tk));
        */
    }
}

fn read_file(buf: &mut impl Buf) -> Result<(), Box<dyn Error>> {
    let (brand, version, compat_brands) = read_ftyp(buf);
    let cb = compat_brands
        .iter()
        .map(|v| match from_utf8(v) {
            Ok(s) => s.to_string(),
            Err(_) => format!("{:?}", v),
        })
        .collect::<Vec<String>>();

    println!(
        "MPEG-4 file: {} - {:?}, {:?}",
        from_utf8(&brand)?,
        version,
        cb
    );
    return read_boxes(buf);
}

fn read_boxes(buf: &mut impl Buf) -> Result<(), Box<dyn Error>> {
    // Read in file type box.

    loop {
        let (size, b_type) = read_box_header(buf);
        // This is some hackery for the Apple ilist
        // box which conatins boxes with
        // boxtype:  0xA9 + <3-byte-string>.
        // I'd like to compare to str in the match.
        // so create a lossy, then grab it from the
        // Cow (copy-on-write smartpointer) that from_utf8_lossy returns.
        // with into_owned() and finally get the str out of the string.
        // I could, and have, compared against the actual [u8,4] but it makes
        // for slightly messy dsecriptions of the tokens.
        // ... and this doesn't quite work as expected in the compares.
        // println!("{}", 0xA9 as char);
        // let sb_type = String::from_utf8_lossy(&b_type);
        // match sb_type.into_owned().as_str() {
        //     "moov" => println!("moov"),
        //     "©alb" => println!("Calbum"),
        //     _ => println!("not moov"),
        // }
        // Do this just for printout.
        println!("{:?}  [{}]", String::from_utf8_lossy(&b_type), size);

        // Keep the compare as [u8;4] for now.
        // It's not that bad, and is probably faster as  it sould
        // be tricial for the compiler to generate efficcient compare
        // code for the [u8;4]s.
        let next;
        match &b_type {
            // Container boxes.
            b"moov" | b"trak" | b"udta" | b"mdia" | b"minf" | b"dinf" | b"ilst" | b"stbl" => {
                let mut b = &buf.bytes()[0..size - 8];
                read_boxes(&mut b)?;
                println!("------  {:?}", from_utf8(&b_type)?);
                next = size - 8;
            }
            // Full Box and container.
            b"meta" => {
                let mut read = 8;
                let (r, v, f) = read_version_flag(buf);
                read += r;
                println!("\tversion: {:?}", v);
                println!("\tflags: {:?}", f);

                let mut b = &buf.bytes()[0..size - (read)];
                read_boxes(&mut b)?;
                println!("------  {:?}", from_utf8(&b_type)?);
                next = size - read;
            } // full box witdth
            b"mvhd" => {
                let mut read = 8;
                let (r, v, f) = read_version_flag(buf);
                read += r;

                println!("\tVersion: {:?}", v);
                println!("\tFlags: {:?}", f);

                // Store seconds since begining of 1904
                let creation; // second in Jan 1, 1904.
                let modification; // second in Jan 1, 1904.
                let timescale; // units in one second.
                let duration; // length in timescale.
                read += if v == 1 {
                    creation = buf.get_u64();
                    modification = buf.get_u64();
                    timescale = buf.get_u32();
                    duration = buf.get_u64();
                    28
                } else {
                    creation = buf.get_u32() as u64;
                    modification = buf.get_u32() as u64;
                    timescale = buf.get_u32();
                    duration = buf.get_u32() as u64;
                    16
                };

                println!("\tCreation: {:?} [{:#018x}]", creation, creation);
                println!(
                    "\tModification: {:?} [{:#018x}]",
                    modification, modification
                );
                println!("\tTimescale: {:?} [{:#018x}]", timescale, timescale);
                println!("\tDuration: {:?} [{:#018x}]", duration, duration);

                let rate = buf.get_u32(); // playback speed.
                read += 4;
                let volume = buf.get_u16();
                read += 2;
                buf.advance(10); // reserved.
                read += 10;
                println!("\tRate: {} [{:#010x}]", rate, rate);
                println!("\tVolume: {} [{:#06x}]", volume, volume);

                let mut matrix: [u8; 36] = [0; 36]; // 4 x 9
                buf.copy_to_slice(&mut matrix);
                read += 36;

                buf.advance(24); //  Quickitime values. (predefined 0 in standard MP4).
                read += 24;

                let next_track_id = buf.get_u32();
                read += 4;
                println!("\tNext Track: {}", next_track_id);

                // next = bytes_left;
                next = size - read;
                // next = 0;
            }
            b"tkhd" => {
                // Flag values.
                // bit 0 = track enabled (disabled if 0)
                // bit 1 = track in movie
                // bit 2 = track in preview.
                // bit 3 = track is aspect ratio. Wdith & Height ar not pxiels, but
                // only and indicatio of the desired aspect ratio.
                let mut read = 8;
                let (r, v, f) = read_version_flag(buf);
                read += r;
                println!("\tVersion: {:?}", v);
                println!("\tFlags: {:?}", f);

                let creation;
                let modification;
                let track_id;
                let reserved;
                let duration;
                read += if v == 1 {
                    creation = buf.get_u64();
                    modification = buf.get_u64();
                    track_id = buf.get_u32();
                    reserved = buf.get_u32();
                    duration = buf.get_u64();
                    32
                } else {
                    creation = buf.get_u32() as u64;
                    modification = buf.get_u32() as u64;
                    track_id = buf.get_u32();
                    reserved = buf.get_u32();
                    duration = buf.get_u32() as u64;
                    20
                };
                println!("\tCreation: {:?} [{:0x}]", creation, creation);
                println!("\tModification: {:?} [{:0x}]", modification, modification);
                println!("\tTrack ID: {:?} [{:0x}]", track_id, track_id);
                println!("\tDuration: {:?} [{:0x}]", duration, duration);
                println!("\tReserved: {:?} [{:0x}]", reserved, reserved);

                let res2 = buf.get_u64();
                read += 8;
                let layer = buf.get_u16();
                read += 2;
                let alt_group = buf.get_u16();
                read += 2;
                let volume = buf.get_u16();
                read += 2;
                let res3 = buf.get_u16();
                read += 2;
                let mut matrix: [u8; 36] = [0; 36];
                buf.copy_to_slice(&mut matrix);
                read += 36;
                let width = buf.get_u32();
                read += 4;
                let height = buf.get_u32();
                read += 4;
                println!("\tReserved 2: {:?} [{:0x}]", res2, res2);
                println!("\tLayer: {:?} [{:0x}]", layer, layer);
                println!("\tAlt Group: {:?} [{:0x}]", alt_group, alt_group);
                println!("\tVolume {:?} [{:0x}]", volume, volume);
                println!("\tRes 3 {:?} [{:0x}]", res3, res3);
                println!("\tWidth {:?} [{:0x}]", width, width);
                println!("\tHeight {:?} [{:0x}]", height, height);

                next = size - read;
            }
            b"mdhd" => {
                let mut read = 8;
                let (r, v, f) = read_version_flag(buf);
                read += r;

                println!("\tversion: {}", v);
                println!("\tflags: {:#010b} = {:#03x}", f, f);

                // Store seconds since begining of 1904
                let creation; // second in Jan 1, 1904.
                let modification; // second in Jan 1, 1904.
                let timescale; // units in one second.
                let duration; // length in timescale.
                read += if v == 1 {
                    creation = buf.get_u64();
                    modification = buf.get_u64();
                    timescale = buf.get_u32();
                    duration = buf.get_u64();
                    28
                } else {
                    creation = buf.get_u32() as u64;
                    modification = buf.get_u32() as u64;
                    timescale = buf.get_u32();
                    duration = buf.get_u32() as u64;
                    16
                };
                println!("\tCreation: {:?} [{:0x}]", creation, creation);
                println!("\tModification: {:?} [{:0x}]", modification, modification);
                println!("\tTimescale: {:?} [{:0x}]", timescale, timescale);
                println!("\tDuration: {:?} [{:0x}]", duration, duration);

                // TODO(jdr): convert this to the proper representation.
                let lang = buf.get_u16(); // ISO-639-2/T language code (MSB = 0, then 3 groups of 5 bits.
                read += 2;

                let qual = buf.get_u16(); // predefined = 0; Quicktime Quality value. Normal = 0;
                read += 2;

                println!("\tLanguage code: {:?} [{:0x}]", lang, lang);
                println!("\tQuicktime Quality: {:?} [{:0x}]", qual, qual);

                println!(
                    "\tSize: {}, Read: {}, Size - Read: {}",
                    size,
                    read,
                    size - read
                );
                if size - read != 0 {
                    eprintln!("Mismatch between expected bytes read and size of box.");
                    eprintln!("Size = {}, Bytes read = {}", size, read);
                }
                next = size - read;
            }
            // FullBox with handler type and name.
            b"hdlr" => {
                let (read, v, f, h_type, name) = read_hdlr(buf);
                println!("\tversion: {:?}", v);
                println!("\tflags: {:?}", f);
                println!("\thandler_type: {:?}", from_utf8(&h_type)?);
                println!("\tname: {:?}", name);
                next = size - read;
            }
            b"smhd" => {
                let mut read = 8;
                let (r, v, f) = read_version_flag(buf);
                read += r;

                println!("\tversion: {}", v);
                println!("\tflags: {:#010b} = {:#03x}", f, f);

                // 0 is center, positive is right; negative is left.
                // This is an 8.8 fixed point number.
                // Supposidly 1.0 is full right, -1.0 is full left
                // but that doesn't use the full range of 8.8.
                let balance = buf.get_i16();
                read += 2;
                let res = buf.get_u16(); // reserved = 0;
                read += 2;

                println!("\tBalance: {}", balance);
                println!("\tReserved: {}", res);

                println!(
                    "\tSize: {}, Read: {}, Size - Read: {}",
                    size,
                    read,
                    size - read
                );
                if size - read != 0 {
                    eprintln!("Mismatch between expected bytes read and size of box.");
                    eprintln!("Size = {}, Bytes read = {}", size, read);
                }

                next = size - read;
            }
            b"dref" => {
                let mut read = 8;
                let (r, v, f) = read_version_flag(buf);
                read += r;

                println!("\tversion: {}", v);
                println!("\tflags: {:#010b} = {:#03x}", f, f);

                let entry_count = buf.get_u32();
                read += 4;

                println!("\tEntry count: {}", entry_count);
                println!(
                    "\tSize: {}, Read: {}, Size - Read: {}",
                    size,
                    read,
                    size - read
                );

                let mut b = &buf.bytes()[0..size - read];
                read_boxes(&mut b)?;

                println!("------ {:?}", "dref");

                next = size - read;
                // next = 0;
            }
            // TODO(jdr): this is certainly wrong
            // It appears that if the flag is 1, you have
            // no string,
            // TODO(jdr): add support for the URN box.
            b"url " => {
                let mut read = 8;
                let (r, v, f) = read_version_flag(buf);
                read += r;
                println!("\tversion: {}", v);
                println!("\tflags: {:#010b} = {:#08x}", f, f);

                if f != 1 {
                    // read the string.
                }

                if size - read != 0 {
                    eprintln!("Mismatch between expected bytes read and size of box.");
                    eprintln!("Size = {}, Bytes read = {}", size, read,);
                }

                next = size - read;
            }
            b"stsd" => {
                let mut read = 8;
                let (r, v, f) = read_version_flag(buf);
                read += r;
                println!("\tversion: {}", v);
                println!("\tflags: {:#010b} = {:#08x}", f, f);

                let descriptions = buf.get_u32();
                let d_len = buf.get_u32();
                let mut d_vis: [u8; 4] = [0; 4];
                buf.copy_to_slice(&mut d_vis);
                read += 12;

                buf.advance(6); // reserved and set to 0.
                read += 6;

                let index = buf.get_u16();
                let qt_audio_enc_version = buf.get_u16();
                let qt_audio_enc_revision = buf.get_u16();
                read += 6;

                let mut qt_audio_enc_vendor: [u8; 4] = [0; 4];
                buf.copy_to_slice(&mut qt_audio_enc_vendor);
                read += 4;

                let channels = buf.get_u16();
                let sample_size = buf.get_u16();
                let qt_audio_compression_id = buf.get_u16();
                let qt_audio_packet_sz = buf.get_u16();
                let sample_rate = buf.get_u32();
                read += 12;

                println!("\tDescriptions {}", descriptions);
                println!("\tDescription Length {}", d_len);
                println!("\tDescription Format {:?}", from_utf8(&d_vis)?);

                println!("\tIndex: {}", index);
                println!("\tChannels: {}", channels);
                println!("\tSample Size: {}", sample_size);
                println!("\tSample Rate: {} {:#08x}", sample_rate, sample_rate);

                println!("\tQT Audio Encoding Version: {}", qt_audio_enc_version);
                println!("\tQT Audio Encoding Revision: {}", qt_audio_enc_revision);
                println!(
                    "\tQT Audio Encoding Vendor: {:?}",
                    from_utf8(&qt_audio_enc_vendor)?
                );
                println!("\tQT Audio Compression ID: {}", qt_audio_compression_id);
                println!("\tQT Audio Packet Size: {}", qt_audio_packet_sz);

                // The spec doesn't seem to talk about this,
                // but we see esds boxes following.
                if size - read != 0 {
                    let mut b = &buf.bytes()[0..size - read];
                    read_boxes(&mut b)?;
                    println!("------ {:?}", from_utf8(&b_type)?);
                    // eprintln!("Mismatch between expected bytes read and size of box.");
                    // eprintln!("Size = {}, Bytes read = {}", size, read,);
                }

                next = size - read;
            }
            b"esds" => {
                let mut read = 8;
                let (r, v, f) = read_version_flag(buf);
                read += r;

                println!("\tversion: {}", v);
                println!("\tflags: {:#010b} = {:#08x}", f, f);

                let d_type = buf.get_u8();
                read += 1;
                println!("\tDescriptor type: {} {:#02x}", d_type, d_type);

                println!("\tESDS not othewise implemented");
/*
                // - types are Start = 0x80 ; End = 0xFE
                let mut d_type_tag: [u8; 3] = [0; 3];
                buf.copy_to_slice(&mut d_type_tag);
                read += 3;

                // ESD
                println!(
                    "\tDescriptor type tag: {:?} {:?}",
                    from_utf8(&d_type_tag),
                    d_type_tag,
                );

                let d_len = buf.get_u8();
                read += 1;
                println!("\tDescriptor length: {} {:#04x}", d_len, d_len);

                let es_id = buf.get_u16();
                read += 2;
                println!("\tES ID: {} {:#06x}", es_id, es_id);

                let stream_prio = buf.get_u8();
                read += 1;
                println!("\tStream Priority: {} {:#02x}", stream_prio, stream_prio);

                let decoder_config_desc = buf.get_u8();
                read += 1;
                println!(
                    "\tDecoder Config Descriptor: {} {:#02x}",
                    decoder_config_desc, decoder_config_desc
                );

                let mut xd_type_tag: [u8; 3] = [0; 3];
                buf.copy_to_slice(&mut xd_type_tag);
                read += 3;
                println!(
                    "\tExtended Descriptor Type: {:?} {:?}",
                    from_utf8(&xd_type_tag),
                    xd_type_tag
                );

                let d_type_leng = buf.get_u8();
                read += 1;
                println!("\tDescriptor type length: {}", d_type_leng);

                // - type IDs are system v1 = 1 ; system v2 = 2
                // - type IDs are MPEG-4 video = 32 ; MPEG-4 AVC SPS = 33
                // - type IDs are MPEG-4 AVC PPS = 34 ; MPEG-4 audio = 64
                // - type IDs are MPEG-2 simple video = 96
                // - type IDs are MPEG-2 main video = 97
                // - type IDs are MPEG-2 SNR video = 98
                // - type IDs are MPEG-2 spatial video = 99
                // - type IDs are MPEG-2 high video = 100
                // - type IDs are MPEG-2 4:2:2 video = 101
                // - type IDs are MPEG-4 ADTS main = 102
                // - type IDs are MPEG-4 ADTS Low Complexity = 103
                // - type IDs are MPEG-4 ADTS Scalable Sampling Rate = 104
                // - type IDs are MPEG-2 ADTS = 105 ; MPEG-1 video = 106
                // - type IDs are MPEG-1 ADTS = 107 ; JPEG video = 108
                // - type IDs are private audio = 192 ; private video = 208
                // - type IDs are 16-bit PCM LE audio = 224 ; vorbis audio = 225
                // - type IDs are dolby v3 (AC3) audio = 226 ; alaw audio = 227
                // - type IDs are mulaw audio = 228 ; G723 ADPCM audio = 229
                // - type IDs are 16-bit PCM Big Endian audio = 230
                // - type IDs are Y'CbCr 4:2:0 (YV12) video = 240 ; H264 video = 241
                // - type IDs are H263 video = 242 ; H261 video = 243
                let object_type = buf.get_u8();
                read += 1;
                println!("\tObject Type: {} {:#02x}", object_type, object_type);

                let flags_buffer_size = buf.get_u32();
                read += 4;

                // - types are Start = 0x80 ; End = 0xFE

                // MSByte of flags_buffer_size
                // 6msb  bits of stream type.
                // 1 bit of upstream flag
                // 1lsb bit reserved flag
                let stream_flags = (flags_buffer_size >> 24) as u8;

                // 24 bits unsigned buffer size (store din u32).
                let buffer_size = (0xFF000000) & flags_buffer_size;

                println!(
                    "\tStream Flags: {:#08b} {:#02x}",
                    stream_flags, stream_flags
                );
                println!("\tBuffer Size: {} {:#06x}", buffer_size, buffer_size);

                let maximum_bit_rate = buf.get_u32();
                read += 4;
                println!("\tMaximum Bit Rate: {}", maximum_bit_rate);

                let average_bit_rate = buf.get_u32();
                read += 4;
                println!("\tAverage Bit Rate: {}", average_bit_rate);

                // This is pretty wierd. I have a test file, which may
                // or may not be valid. That essentially ends here.
                // so:
                let decoder_type = buf.get_u8();
                read += 1;
                println!("\tDecoder Type:  {:#010x}", decoder_type);

                println!(
                    "Size: {}, Read: {}, Left: {}, buf.remaining {}",
                    size,
                    read,
                    size - read,
                    buf.remaining()
                );
                if (size - read) > 0 && buf.remaining() >= (size - read) {
                    let type_length = buf.get_u8() as usize; // as usize for how we use it below.
                    read += 1;
                    println!("\tType Length: {}", type_length);

                    // read type length bytes for ES Start header start code.
                    buf.advance(type_length);
                    read += type_length;

                    // MSB = SL descriptor type tag
                    // 3 LSB = SL descriptor type string hex value.
                    let sl_config_type = buf.get_u32();
                    read += 4;
                    println!(
                        "\tSL Config type: {} {:#06x}",
                        sl_config_type, sl_config_type
                    );

                    let sl_type_length = buf.get_u8();
                    read += 1;
                    println!("\tSL Type Length: {}", sl_type_length);

                    let sl_value = buf.get_u8();
                    read += 1;
                    println!("\tSL Type Value: {} {:#02x}", sl_value, sl_value);

                    // Check for reading the full box.
                    // alert otherwise.
                    if size - read != 0 {
                        eprintln!("Mismatch between bytes read and size of box.");
                        eprintln!("Size = {}, Bytes read = {}", size, read,);
                    }
                }
*/
                next = size - read;

            }
            b"stts" => {
                let mut read = 8;
                let (r, v, f) = read_version_flag(buf);
                read += r;
                println!("\tversion: {}", v);
                println!("\tflags: {:#010b} = {:#03x}", f, f);

                let num_times = buf.get_u32() as usize;
                read += 4;
                println!("\tNumber of frame rate calculation blocks: {}", num_times);
                if num_times == 1 {
                    let frame_count = buf.get_u32();
                    read += 4;
                    let duration = buf.get_u32();
                    read += 4;

                    println!("\tFixed frame rate, frame count: {}", frame_count);
                    println!("\tFixed frame rate, duration : {}", duration);
                } else {
                    // TODO(jdr) implement variable framerate reporting.
                    buf.advance(num_times * 8);
                    read += num_times * 8;
                }
                if size - read != 0 {
                    eprintln!("Mismatch between bytes read and size of box.");
                    eprintln!(
                        "Size = {}, Bytes read = {}, difference: {}",
                        size,
                        read,
                        size - read
                    );
                }

                next = size - read;
            }
            b"stsc" => {
                let mut read = 8;
                let (r, v, f) = read_version_flag(buf);
                read += r;
                println!("\tversion: {}", v);
                println!("\tflags: {:#010b} = {:#03x}", f, f);

                let num_blocks = buf.get_u32();
                read += 4;

                println!("\tNumber of chunks: {}", num_blocks);

                let mut chunks = Vec::new();
                for _ in 0..num_blocks {
                    let first_chunk = buf.get_u32();
                    let samples_per_chunk = buf.get_u32();
                    let sample_description_index = buf.get_u32();
                    chunks.push((first_chunk, samples_per_chunk, sample_description_index));
                    read += 12;
                }
                println!(
                    "\tChunks: {:?}",
                    chunks
                        .iter()
                        .map(|v| {
                            let (f, s, i) = v;
                            format!("first: {}, samples: {}, description index: {}", f, s, i)
                        })
                        .collect::<Vec<String>>()
                );

                if size - read != 0 {
                    eprintln!("Mismatch between bytes read and size of box.");
                    eprintln!(
                        "Size = {}, Bytes read = {}, difference: {}",
                        size,
                        read,
                        size - read
                    );
                }

                next = size - read;
            }
            b"stsz" => {
                let mut read = 8;
                let (r, v, f) = read_version_flag(buf);
                read += r;
                println!("\tversion: {}", v);
                println!("\tflags: {:#010b} = {:#03x}", f, f);

                let block_sample_size = buf.get_u32();
                read += 4;

                let num_block_sizes = buf.get_u32();
                read += 4;

                println!("\tBlock Byte Size: {}", block_sample_size);
                println!("\tNumber of block sizes: {}", num_block_sizes);

                let mut sizes = Vec::new();
                if block_sample_size == 0 {
                    for _ in 0..num_block_sizes {
                        sizes.push(buf.get_u32());
                        read += 4;
                    }
                }

                // sizes.sort();
                // println!("Sizes: {:?}", sizes);
                println!("\tSizes: Not Displayed.");

                if size - read != 0 {
                    eprintln!("Mismatch between bytes read and size of box.");
                    eprintln!(
                        "Size = {}, Bytes read = {}, difference: {}",
                        size,
                        read,
                        size - read
                    );
                }

                next = size - read;
            }
            b"stco" => {
                let mut read = 8;
                let (r, v, f) = read_version_flag(buf);
                read += r;
                println!("\tversion: {}", v);
                println!("\tflags: {:#010b} = {:#03x}", f, f);

                let entry_count = buf.get_u32();
                read += 4;
                println!("\tFile offset entry count: {}", entry_count);

                let mut offsets = Vec::new();
                for _ in 00..entry_count {
                    offsets.push(buf.get_u32());
                    read += 4;
                }
                // println!("\tFile offests: {:?}", offsets);
                println!("\tChunk file offsets: Not Displayed");

                if size - read != 0 {
                    eprintln!("Mismatch between bytes read and size of box.");
                    eprintln!(
                        "Size = {}, Bytes read = {}, difference: {}",
                        size,
                        read,
                        size - read
                    );
                }

                next = size - read;
            }
            // Data Boxes for ilst metadata.
            [0xa9, b'a', b'l', b'b']
            | [0xa9, b'a', b'r', b't']
            | [0xa9, b'A', b'R', b'T']
            | [0xa9, b'c', b'm', b't']  // Comment
            | [0xa9, b'd', b'a', b'y']  // Year
            | [0xa9, b'g', b'e', b'n']  // Genre
            | [0xa9, b'g', b'r', b'p']  // Genre
            | [0xa9, b'l', b'y', b'r']  // Lyric
            | [0xa9, b'n', b'a', b'm']  // Title
            | [0xa9, b't', b'o', b'o']  // Encoder
            | [0xa9, b'w', b'r', b't']
            | b"aART"
            | b"covr"
            | b"cpil"
            | b"disk"
            | b"gnre"
            | b"pgap"
            | b"tmpo"
            | b"trkn" => {
                let (v, f, data) = read_data_box(buf);
                match data {
                    DataBoxContent::Data(d) => match &b_type {
                        b"disk" => {
                            let disk = u16::from_be_bytes([d[2], d[3]]);
                            let disks = u16::from_be_bytes([d[4], d[5]]);
                            println!("\tDisk: {}", disk);
                            println!("\tDisks: {}", disks);
                            println!("\tContents: {:?}", d);
                        }
                        b"trkn" => {
                            let track = u16::from_be_bytes([d[2], d[3]]);
                            let tracks = u16::from_be_bytes([d[4], d[5]]);
                            println!("\tTrack: {}", track);
                            println!("\tTracks: {}", tracks);
                            println!("\tContents: {:?}", d);
                        }
                        _ => {
                            let mut pb = d.as_slice();
                            if d.len() > 32 {
                                pb = &d[0..32];
                            }
                            println!("\tDataBox::Data");
                            println!("\tLength: {}", d.len());
                            println!("\tContent: {:?}", pb);
                        }
                    },
                    DataBoxContent::Text(s) => {
                        println!("\tDataBoxContent::Text");
                        println!("\tValue: {:?}", s);
                    }
                    DataBoxContent::Byte(b) => {
                        println!("\tDataBoxContent::Byte");
                        eprintln!("\tValue: {:?}", b);
                    }
                }
                println!("\tversion: {}", v);
                println!("\tflags: {:#010b} = {:#03x}", f, f);
                next = 0;
            }
            b"----" => {
                let (mean, name, data) = read_apple_info_box(buf);
                println!("\tApple Additional Info Box");
                println!("\tMean Value: {:?}", mean);
                println!("\tName Value: {:?}", name);
                println!("\tValue: {:?}", data);
                next = 0;
            }
            b"data" => {
                let (v, f, data) = read_data(buf, size);
                println!("\tversion: {:?}", v);
                println!("\tflags: {:#010b} = {:024x}", f, f);
                println!("\tContents: {:?}", data);
                next = 0; //size - read;
            }
            b"free" | b"skip" => {
                println!("\t{} Byte of free space in the current box.", size);
                next = size - 8;
            }
            b"wide" => {
                println!(
                    "\t{} Bytes of free space to lengthen (widden) the file box.",
                    size
                );
                next = size - 8;
            }
            b"mdat" => {
                println!("\tMedia data. {} bytes", size);
                next = size - 8;
            }
            _ => {
                println!("\tDefault.");
                match from_utf8(&b_type) {
                    Ok(s) => println!("\tFound type: {}", s),
                    Err(_) => println!("\tFound type: {:?}", b_type),
                };
                // println!("\t * Found type: {}", from_utf8(&b_type)?);
                next = size - 8;
            }
        }

        // TODO(jdr) If size == 0, box goes to end of file.
        // println!("\tNext is: {}", next);
        // println!("\tRemaining is: {}", buf.remaining());
        // println!("\tRemaining - Next: {}", buf.remaining() - next);

        if buf.remaining() <= next {
            break;
        }
        buf.advance(next);
    }

    Ok(())
}

// TODO(jdr) - this will fail on 32 bit machines.
// because we're reading in 64 bit size values as usize
// which will wrap on a 32 bit machine.
// Read Size and box type.
// Read 4 bytes of size (big endian)
// Size is the number of bytes in the box
// including all fields and contained
// boxes.
// Read 4 bytes of box type.
// Read 4 bytes of the version and flags.
// Returns the Version, Flags, Size, and the Type of the box.
fn read_box_header(buf: &mut impl Buf) -> (usize, [u8; 4]) {
    let ss = buf.get_u32();
    let mut b_type: [u8; 4] = [0; 4];
    buf.copy_to_slice(&mut b_type);
    let size: usize;
    if ss == 1 {
        size = buf.get_u64() as usize;
    } else {
        size = ss as usize;
    }
    return (size, b_type);
}

// fn read_box_header(buf: &mut impl Buf) -> (u8, u32, usize, [u8; 4]) {
//     let (size, b_type) = read_box_size_type(buf);
//     let (v, f) = read_version_flag(buf);
//     return (v, f, size, b_type);
// }

// Reads 4 bytes
// Return the top byte as the version
// number in a u8.
// Returns the bottom three bytes as the flags in a u32.
fn read_version_flag(buf: &mut impl Buf) -> (usize, u8, u32) {
    let mut f = buf.get_u32();
    let v = (f >> 28) as u8; // High byte is the version
    f &= 0x00FFFFFF; // bottom three bytes are the flags.
    return (4, v, f);
}

// TODO(jdr): check for the ftyp and return an error if not found?
/// Read an ftyp box contents.
/// returns the major brand, the minor version, and the compatible brands.
fn read_ftyp(buf: &mut impl Buf) -> ([u8; 4], u32, Vec<[u8; 4]>) {
    // println!("Calling read_ftyp");
    let (size, t) = read_box_header(buf); // t = fytp.
    let mut brand: [u8; 4] = [0; 4];
    buf.copy_to_slice(&mut brand);
    let version = buf.get_u32();

    let mut left = size - 16;

    let mut c_brands = Vec::<[u8; 4]>::new();
    while left > 0 {
        let mut cb: [u8; 4] = [0; 4];
        buf.copy_to_slice(&mut cb);
        c_brands.push(cb);
        left -= 4;
    }
    return (brand, version, c_brands);
}

fn read_hdlr(buf: &mut impl Buf) -> (usize, u8, u32, [u8; 4], String) {
    let mut read = 8;
    let (r, v, f) = read_version_flag(buf);
    read += r;

    let _ = buf.get_u32(); // predefined as 0.
    read += 4;

    let mut handler_type: [u8; 4] = [0; 4];
    buf.copy_to_slice(&mut handler_type);
    read += 4;

    let mut name = String::new();
    let mut c;
    loop {
        c = buf.get_u8();
        if c == b'\0' {
            break;
        }
        name.push(c as char);
    }
    read += name.len() + 1;

    (read, v, f, handler_type, name)
}

// TODO(jdr): return the raw bytes
// if we don't get a good flag value.
#[derive(Debug)]
pub enum DataBoxContent {
    Text(String),
    Data(Vec<u8>),
    Byte(u8),
}

const IMPLICIT_FLAGS: u32 = 0;
const TEXT_FLAGS: u32 = 1;
const JPEG_FLAGS: u32 = 13;
const PNG_FLAGS: u32 = 14;
const BYTE_FLAGS: u32 = 21;

// TODDO(jdr): Check for flag-type size mismatch errors and return them.
// E.g. When the flags say BYTE_FLAGS, but size says there is more than one
// byte left to read.
// TOOD(jdr): Add support for the image types outside of just a data array.
fn read_data(buf: &mut impl Buf, size: usize) -> (u8, u32, DataBoxContent) {
    let (_, v, f) = read_version_flag(buf);

    let _ = buf.get_u32(); // predefined as 0.

    let value;
    match f {
        TEXT_FLAGS => {
            let mut sv = String::new();
            let mut c;
            for _ in 8..(size - 8) {
                c = buf.get_u8();
                sv.push(c as char);
            }
            value = DataBoxContent::Text(sv);
        }
        IMPLICIT_FLAGS | JPEG_FLAGS | PNG_FLAGS => {
            let mut dv = Vec::<u8>::new();
            for _ in 8..(size - 8) {
                dv.push(buf.get_u8());
            }
            value = DataBoxContent::Data(dv);
        }
        BYTE_FLAGS => {
            value = DataBoxContent::Byte(buf.get_u8());
            // 8 bytes of size + 'data' header,
            // 4 bytes v/flag, 4 bytes of reserve, 1 byte of returned data.
            buf.advance(size - 17);
        }
        _ => {
            eprintln!("Unknown 'data' Box flag value: {}", f);
            let mut dv = Vec::<u8>::new();
            for _ in 8..(size - 8) {
                dv.push(buf.get_u8());
            }
            value = DataBoxContent::Data(dv);
        }
    }
    return (v, f, value);
}

fn read_apple_info_box(buf: &mut impl Buf) -> (String, String, DataBoxContent) {
    let (s, t) = read_box_header(buf);
    let (r, v, f) = read_version_flag(buf);
    if &t != b"mean" {
        eprintln!(
            "Expected box type {:?}, got: {:}",
            "mean",
            from_utf8(&t).unwrap()
        );
        let (_, _, d) = read_data(buf, s);
        return ("".to_string(), "".to_string(), d);
    }

    // let (v, f) = read_version_flag(buf);
    let mut mean_val = String::new();
    for _ in 0..(s - 12) {
        mean_val.push(buf.get_u8() as char)
    }

    let (nb_s, t) = read_box_header(buf); // this read 4 + 4 = 8 bytes
    match &t {
        b"name" => {
            let (_, nb_v, nb_f) = read_version_flag(buf); // this read 4 bytes of v/f. set to 0/0
            let mut name_val = String::new();
            for _ in 0..(nb_s - 12) {
                name_val.push(buf.get_u8() as char);
            }
            // println!("\tName box size: {}", nb_s);
            let (v, _, d) = read_data_box(buf);
            return (mean_val, name_val, d);
            // let (nb_v, nb_f, data) = read_data(buf, name_box_s);
        }
        _ => {
            println!(
                "Unknown inner box type. Expected {:?}, got: {:?}",
                "name",
                from_utf8(&t).unwrap()
            );
            let (_, _, d) = read_data(buf, nb_s);
            return (mean_val, "".to_string(), d);
        }
    }
}

fn read_data_box(buf: &mut impl Buf) -> (u8, u32, DataBoxContent) {
    let (s, t) = read_box_header(buf);
    match &t {
        b"data" => {
            return read_data(buf, s);
        }
        _ => {
            eprintln!("ERROR: EXPECTED A DATA BLOCK: Got: {:?}", t);
            eprintln!("name: {:?}", from_utf8(&t).unwrap());
            return read_data(buf, s);
        }
    };
}
