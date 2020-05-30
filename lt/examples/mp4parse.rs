use lt::mpeg4::util;
use mp4parse;
use std::fs::File;

fn main() -> Result<(), std::io::Error> {
    let files = vec![
        "/Volumes/London Backups/Music/iTunes/iTunes Media/Music/Counting Crows/Across A Wire_ Live In New York City/1-01 Round Here.m4a",
        "/Volumes/London Backups/Itunes_Library/The Beatles/Abbey Road/01 Come Together.m4a",
        "/Users/david.rivas/Development/rust/mp4parse-rust/mp4parse/tests/metadata_gnre.mp4"
        // "/Volumes/Audio/TorrentTracks/Leon Russell/Leon Russell 1970-11-20 Fillmore East, NY, NY SBD Flac16/LRUSSELL FE 11 20 70 01.flac",
        // "/Volumes/London Backups/iTunes_Library/ZZ Top/Comb 1_2/03 Track 03.mp3",
        //  "/Volumes/Audio/HDTracks/Keith Jarrett/The Köln Concert/1 Köln, January 24, 1975, Part I.wav",
    ];
    for f in files {
        println!("\nFile: {:?}", f);
        let mut file = File::open(f)?;
        // let t = file::identify(&mut file)?;
        let mut mc = mp4parse::MediaContext::new();
        match mp4parse::read_mp4(&mut file, &mut mc) {
            Ok(_) => (),
            Err(e) => eprintln!("Failed to parse mp4 file: {:?}", e),
        }

        // println!("MediaContext: {:?}", mc);
        let mut tabs = util::Tabs::new();
        println!("No of Tracks: {}", mc.tracks.len());
        let mut i = 1;
        for t in &mc.tracks {
            tabs.indent();
            println!("Track: {}", i);
            i += 1;
            println!("{}Track Type: {:?}", tabs, t.track_type);
            println!("{}EmptyDuration: {:?}", tabs, t.empty_duration);
            println!("{}Media_Time: {:?}", tabs, t.media_time);
            println!("{}TimeScale: {:?}", tabs, t.timescale);
            println!("{}Duration: {:?}", tabs, t.duration);
            println!("{}Track ID: {:?}", tabs, t.track_id.unwrap_or_default());
            println!("{}Track Header Box [tkhd]: {:?}", tabs, t.tkhd);
            if let Some(sd) = t.stsd.as_ref() {
                println!("{}Sample Descriptions [stsd][_]:", tabs);
                tabs.indent();
                for sd in &sd.descriptions {
                    match sd {
                        mp4parse::SampleEntry::Audio(ase) => {
                            println!("{}Audio Sample Entry:", tabs);
                            tabs.indent();
                            println!("{}Codec Type: {:?}", tabs, ase.codec_type);
                            println!("{}Channel Count {:?}", tabs, ase.channelcount);
                            println!("{}Sample Size {:?}", tabs, ase.samplesize);
                            println!("{}Sample Rate {:?}", tabs, ase.samplerate);
                            println!("{}AudioCodecSpecific:", tabs);
                            tabs.indent();
                            match &ase.codec_specific {
                                mp4parse::AudioCodecSpecific::ES_Descriptor(esd) => {
                                    println!("{}ES Desecriptor", tabs);
                                    tabs.indent();
                                    println!("{}Codec: {:?}", tabs, esd.audio_codec);
                                    println!(
                                        "{}Object Type: {:?}",
                                        tabs,
                                        esd.audio_object_type.unwrap_or_default()
                                    );
                                    println!(
                                        "{}Extened Object Type: {:?}",
                                        tabs,
                                        esd.extended_audio_object_type.unwrap_or_default()
                                    );
                                    println!(
                                        "{}Sample Rate: {:?}",
                                        tabs,
                                        esd.audio_sample_rate.unwrap_or_default()
                                    );
                                    println!(
                                        "{}Channel Count: {:?}",
                                        tabs,
                                        esd.audio_channel_count.unwrap_or_default()
                                    );
                                    println!("{}Codec ESDS: {:?}", tabs, esd.codec_esds);
                                    println!(
                                        "{}Codec specific data: {:?}",
                                        tabs, esd.decoder_specific_data
                                    );
                                    tabs.outdent();
                                }
                                mp4parse::AudioCodecSpecific::FLACSpecificBox(fsb) => {
                                    println!("{}{:?}", tabs, fsb)
                                }
                                mp4parse::AudioCodecSpecific::OpusSpecificBox(osb) => {
                                    println!("{}{:?}", tabs, osb)
                                }
                                mp4parse::AudioCodecSpecific::ALACSpecificBox(asb) => {
                                    println!("{}{:?}", tabs, asb)
                                }
                                mp4parse::AudioCodecSpecific::MP3 => {
                                    println!("{}{:?}", tabs, "MP3")
                                }
                                mp4parse::AudioCodecSpecific::LPCM => {
                                    println!("{}{:?}", tabs, "LPCM")
                                }
                            }
                            tabs.outdent();
                        }
                        mp4parse::SampleEntry::Video(vse) => println!("{}{:?}", tabs, vse),
                        _ => (),
                    }
                    tabs.outdent();
                }
                tabs.outdent();
            } else {
                println!("{}Sample Description Box: None", tabs);
            }
            if let Some(stts) = t.stts.as_ref() {
                println!("{}Time To Sample [stts]:", tabs);
                tabs.indent();
                println!("{}STTS: {:?}", tabs, stts);
                tabs.outdent();
            }
            if let Some(stsc) = t.stsc.as_ref() {
                println!("{}Sample To Chunk [stsc]:", tabs);
                tabs.indent();
                println!("{}STSC: {:?}", tabs, stsc);
                tabs.outdent();
            }
            if let Some(_stsz) = t.stsz.as_ref() {
                println!("{}Sample Size [stsz]:", tabs);
                tabs.indent();
                println!("{}Big array of ints", tabs);
                // println!("{}STSC: {:?}", tabs, stsz);
                tabs.outdent();
            }
            if let Some(_stco) = t.stco.as_ref() {
                println!("{}Chunk Offset [stco]:", tabs);
                tabs.indent();
                println!("{}Big array of ints.", tabs);
                // println!("{}STCO: {:?}", tabs, stco);
                tabs.outdent();
            }
            if let Some(stss) = t.stss.as_ref() {
                println!("{}Sync Sample: [stss]:", tabs);
                tabs.indent();
                println!("{}STSS: {:?}", tabs, stss);
                tabs.outdent();
            }
            print!("{}Composition Offset [ctts]: ", tabs);
            match t.ctts.as_ref() {
                Some(ctts) => {
                    tabs.indent();
                    println!("{}CTTS: {:?}", tabs, ctts);
                    tabs.outdent();
                }
                None => println!("None"),
            };
        }
        tabs.outdent();
        println!("{}Movie Extends [mvex]: {:?}", tabs, mc.mvex);
        println!(
            "{}Protection System Specific Header [pssh]: {:?}",
            tabs, mc.psshs
        );
        println!("{}User data [udta]", tabs);
        if let Some(Ok(ud)) = mc.userdata {
            if let Some(md) = ud.meta {
                tabs.indent();
                // println!("{}Album: {:?}", tabs, String::from_utf8_lossy(md.album.unwrap_or_default("None")));
                println!("{}Album: {:?}", tabs, to_string(&md.artist));
                println!("{}Album Artist: {:?}", tabs, to_string(&md.album_artist));
                println!("{}Artist: {:?}", tabs, to_string(&md.album));
                println!("{}Comment: {:?}", tabs, to_string(&md.comment));
                println!("{}Year: {:?}", tabs, to_string(&md.year));
                println!("{}Title: {:?}", tabs, to_string(&md.title));
                println!("{}Genre: {:?}", tabs, md.genre);
                println!("{}Track Number: {:?}", tabs, md.track_number);
                println!("{}Disk Number: {:?}", tabs, md.disc_number);
                println!("{}Total Tracks: {:?}", tabs, md.total_tracks);
                println!("{}Total Disks: {:?}", tabs, md.total_discs);
                println!("{}Composer: {:?}", tabs, to_string(&md.composer));
                println!("{}Encoder: {:?}", tabs, to_string(&md.encoder));
                println!("{}Encoded by:: {:?}", tabs, to_string(&md.encoded_by));
                println!("{}BPM by:: {:?}", tabs, md.beats_per_minute);
                println!("{}Copyright by:: {:?}", tabs, to_string(&md.copyright));
                println!("{}Compilation:: {:?}", tabs, md.compilation);
                println!("{}Advisory rating:: {:?}", tabs, md.advisory);
                println!("{}Rating:: {:?}", tabs, to_string(&md.rating));
                println!("{}Grouping: {:?}", tabs, to_string(&md.grouping));
                println!("{}Media Type: {:?}", tabs, md.media_type);
                println!("{}Podcast: {:?}", tabs, md.podcast);
                println!("{}Category: {:?}", tabs, to_string(&md.category));
                println!("{}Keyword: {:?}", tabs, to_string(&md.keyword));
                println!("{}Podcast URL: {:?}", tabs, to_string(&md.podcast_url));
                println!("{}Podcast GUID: {:?}", tabs, to_string(&md.podcast_guid));
                println!("{}Description: {:?}", tabs, to_string(&md.description));
                println!(
                    "{}Long Desecription: {:?}",
                    tabs,
                    to_string(&md.long_description)
                );
                println!("{}Lyrics: {:?}", tabs, to_string(&md.lyrics));
                println!(
                    "{}TV Network Name: {:?}",
                    tabs,
                    to_string(&md.tv_network_name)
                );
                println!("{}TV Show Name: {:?}", tabs, to_string(&md.tv_show_name));
                println!(
                    "{}TV Episode Name: {:?}",
                    tabs,
                    to_string(&md.tv_episode_name)
                );
                println!("{}TV Episode Number: {:?}", tabs, &md.tv_episode_number);
                println!("{}TV Season: {:?}", tabs, &md.tv_season);
                println!("{}Purchase Date: {:?}", tabs, to_string(&md.purchase_date));
                println!("{}Gapless playback {:?}", tabs, &md.gapless_playback);
                match md.cover_art {
                    Some(_) => println!("{}Cover Art: Yes", tabs),
                    None => println!("{}Cover Art: No", tabs),
                }
                println!("{}Owner {:?}", tabs, to_string(&md.owner));
                println!("{}HD Video: {:?}", tabs, &md.hd_video);
                println!("{}Sort Name{:?}", tabs, to_string(&md.sort_name));
                println!("{}Sort Album{:?}", tabs, to_string(&md.sort_album));
                println!("{}Sort Artist{:?}", tabs, to_string(&md.sort_artist));
                println!(
                    "{}Sort Album Artist{:?}",
                    tabs,
                    to_string(&md.sort_album_artist)
                );
                println!("{}Sort Composer{:?}", tabs, to_string(&md.sort_composer));
            }
        }
    }
    Ok(())
}

fn to_string(os: &Option<mp4parse::TryString>) -> String {
    match os.as_ref() {
        Some(v) => String::from_utf8_lossy(v.as_slice()).to_string(),
        None => "None".to_string(),
    }
}
