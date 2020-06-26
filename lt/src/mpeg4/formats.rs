extern crate num_derive;
extern crate num_traits;

/// Audio Object Type Table:
/// 0: Null
/// 1: AAC Main
/// 2: AAC LC (Low Complexity)
/// 3: AAC SSR (Scalable Sample Rate)
/// 4: AAC LTP (Long Term Prediction)
/// 5: SBR (Spectral Band Replication)
/// 6: AAC Scalable
/// 7: TwinVQ
/// 8: CELP (Code Excited Linear Prediction)
/// 9: HXVC (Harmonic Vector eXcitation Coding)
/// 10: Reserved
/// 11: Reserved
/// 12: TTSI (Text-To-Speech Interface)
/// 13: Main Synthesis
/// 14: Wavetable Synthesis
/// 15: General MIDI
/// 16: Algorithmic Synthesis and Audio Effects
/// 17: ER (Error Resilient) AAC LC
/// 18: Reserved
/// 19: ER AAC LTP
/// 20: ER AAC Scalable
/// 21: ER TwinVQ
/// 22: ER BSAC (Bit-Sliced Arithmetic Coding)
/// 23: ER AAC LD (Low Delay)
/// 24: ER CELP
/// 25: ER HVXC
/// 26: ER HILN (Harmonic and Individual Lines plus Noise)
/// 27: ER Parametric
/// 28: SSC (SinuSoidal Coding)
/// 29: PS (Parametric Stereo)
/// 30: MPEG Surround
/// 31: (Escape value)
/// 32: Layer-1
/// 33: Layer-2
/// 34: Layer-3
/// 35: DST (Direct Stream Transfer)
/// 36: ALS (Audio Lossless)
/// 37: SLS (Scalable LosslesS)
/// 38: SLS non-core
/// 39: ER AAC ELD (Enhanced Low Delay)
/// 40: SMR (Symbolic Music Representation) Simple
/// 41: SMR Main
/// 42: USAC (Unified Speech and Audio Coding) (no SBR)
/// 43: SAOC (Spatial Audio Object Coding)
/// 44: LD MPEG Surround
/// 45: USAC
#[derive(FromPrimitive, ToPrimitive)]
pub enum AudioObjectTypes {
    Null = 0,
    AAC,    // Main
    AACLC,  //(Low Complexity)
    AACSSR, // (Scalable Sample Rate)
    AACLTP, // (Long Term Prediction)
            /*
            5: SBR (Spectral Band Replication)
            6: AAC Scalable
            7: TwinVQ
            8: CELP (Code Excited Linear Prediction)
            9: HXVC (Harmonic Vector eXcitation Coding)
            10: Reserved
            11: Reserved
            12: TTSI (Text-To-Speech Interface)
            13: Main Synthesis
            14: Wavetable Synthesis
            15: General MIDI
            16: Algorithmic Synthesis and Audio Effects
            17: ER (Error Resilient) AAC LC
            18: Reserved
            19: ER AAC LTP
            20: ER AAC Scalable
            21: ER TwinVQ
            22: ER BSAC (Bit-Sliced Arithmetic Coding)
            23: ER AAC LD (Low Delay)
            24: ER CELP
            25: ER HVXC
            26: ER HILN (Harmonic and Individual Lines plus Noise)
            27: ER Parametric
            28: SSC (SinuSoidal Coding)
            29: PS (Parametric Stereo)
            30: MPEG Surround
            31: (Escape value)
            32: Layer-1
            33: Layer-2
            34: Layer-3
            35: DST (Direct Stream Transfer)
            36: ALS (Audio Lossless)
            37: SLS (Scalable LosslesS)
            38: SLS non-core
            39: ER AAC ELD (Enhanced Low Delay)
            40: SMR (Symbolic Music Representation) Simple
            41: SMR Main
            42: USAC (Unified Speech and Audio Coding) (no SBR)
            43: SAOC (Spatial Audio Object Coding)
            44: LD MPEG Surround
            45: USAC
            */
}

impl std::fmt::Display for AudioObjectTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            AudioObjectTypes::Null => f.write_str("Unknown")?,
            AudioObjectTypes::AAC => f.write_str("AAC Main")?,
            AudioObjectTypes::AACLC => f.write_str("AAC Low Complexity")?,
            AudioObjectTypes::AACSSR => f.write_str("AAC Scalable Sample Rate")?,
            AudioObjectTypes::AACLTP => f.write_str("AAC Long Term Prediction")?,
        };
        Ok(())
    }
}

impl std::fmt::Debug for AudioObjectTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            AudioObjectTypes::Null => f.write_str("Unknown")?,
            AudioObjectTypes::AAC => f.write_str("AAC Main")?,
            AudioObjectTypes::AACLC => f.write_str("AAC Low Complexity")?,
            AudioObjectTypes::AACSSR => f.write_str("AAC Scalable Sample Rate")?,
            AudioObjectTypes::AACLTP => f.write_str("AAC Long Term Prediction")?,
        };
        Ok(())
    }
}

impl From<u8> for AudioObjectTypes {
    fn from(v: u8) -> Self {
        match v {
            v if v == AudioObjectTypes::Null as u8 => AudioObjectTypes::Null,
            v if v == AudioObjectTypes::AAC as u8 => AudioObjectTypes::AAC,
            v if v == AudioObjectTypes::AACLC as u8 => AudioObjectTypes::AACLC,
            _ => AudioObjectTypes::Null,
        }
    }
}

impl Default for AudioObjectTypes {
    fn default() -> Self {
        AudioObjectTypes::Null
    }
}

/*
/// Channel Configurations
/// 0: Defined in AOT Specifc Config
/// 1: 1 channel: front-center
/// 2: 2 channels: front-left, front-right
/// 3: 3 channels: front-center, front-left, front-right
/// 4: 4 channels: front-center, front-left, front-right, back-center
/// 5: 5 channels: front-center, front-left, front-right, back-left, back-right
/// 6: 6 channels: front-center, front-left, front-right, back-left, back-right, LFE-channel
/// 7: 8 channels: front-center, front-left, front-right, side-left, side-right, back-left, back-right, LFE-channel
/// 8-15: Reserved
*/
