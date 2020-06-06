extern crate nom;
// use lt::file;
use nom::bytes::complete::tag;
use std::fs::File;
use std::io::Read;

fn main() -> Result<(), std::io::Error> {
    println!("{:?}", hello_parser("world, hello"));
    // println!("{:?}", hello_parser("hello world"));
    // println!("{:?}", hello_parser("hello world again"));

    let files = vec![
        "/Volumes/London Backups/Music/iTunes/iTunes Media/Music/Counting Crows/Across A Wire_ Live In New York City/1-01 Round Here.m4a",
        "/Volumes/Audio/TorrentTracks/Leon Russell/Leon Russell 1970-11-20 Fillmore East, NY, NY SBD Flac16/LRUSSELL FE 11 20 70 01.flac",
        "/Volumes/London Backups/iTunes_Library/ZZ Top/Comb 1_2/03 Track 03.mp3",
         "/Volumes/Audio/HDTracks/Keith Jarrett/The Köln Concert/1 Köln, January 24, 1975, Part I.wav",
    ];
    for f in files {
        let mut file = File::open(f)?;
        let md = file.metadata()?;
        let mut vbuf = Vec::<u8>::with_capacity(md.len() as usize);
        let n = file.read_to_end(&mut vbuf)?;
        println!("{} is {} bytes long", f, n);
        let buf = vbuf.as_slice();

        match find_ftyp(&buf[4..8]) {
            Ok((r, m)) => eprintln!("MP4: found {:?}, {} bytes left", m, r.len()),
            // Err(e) => eprintln!("Err: {}", e),
            Err(e) => match e {
                nom::Err::Incomplete(n) => eprintln!("Needed: {:?}", n),
                nom::Err::Error((_, k)) => match k {
                    nom::error::ErrorKind::Tag => eprintln!("Didnt find a match for ftyp"),
                    _ => eprintln!("Errorkind: {:?}", k),
                },
                nom::Err::Failure(fe) => eprintln!("Failure {:?}", fe),
            },
        };
        // let t = file::identify(&mut file)?;

        // println!("{}\n\t{:?}", f, t);
    }
    Ok(())
}

fn hello_parser(i: &str) -> nom::IResult<&str, &str> {
    tag("hello")(i)
}

fn find_ftyp(b: &[u8]) -> nom::IResult<&[u8], &[u8]> {
    tag(b"ftyp")(b)
}
