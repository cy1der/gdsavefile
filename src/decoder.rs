use base64::engine::{general_purpose::URL_SAFE, Engine};
use flate2::read::GzDecoder;
use rfd::FileDialog;
use std::io::{Error, ErrorKind};
use std::result::Result::Err;
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};
use xml::{
    reader::{Error as XmlReaderError, ParserConfig},
    writer::EmitterConfig,
    EventReader, EventWriter,
};

pub struct FileSelected {
    pub file_path: PathBuf,
    pub data: Vec<u8>,
}

impl FileSelected {
    fn new(file_path: PathBuf) -> Self {
        FileSelected {
            file_path,
            data: Vec::new(),
        }
    }

    fn open_file(&self) -> Option<Vec<u8>> {
        let mut savedata: Vec<u8> = Vec::new();
        if let Ok(mut file) = File::open(&self.file_path) {
            if file.read_to_end(&mut savedata).is_ok() {
                return Some(savedata);
            }
        }
        None
    }

    fn file_name(&self) -> Option<&str> {
        self.file_path.file_stem().and_then(|name| name.to_str())
    }

    fn decrypt_xor(data: &[u8], key: u8) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::with_capacity(data.len());
        for byte in data {
            result.push(byte ^ key);
        }
        result
    }

    fn remove_null_bytes(vector: &mut Vec<u8>) {
        while let Some(byte) = vector.last() {
            if *byte == 0 {
                vector.pop();
            } else {
                break;
            }
        }
    }

    pub fn decode(&mut self) -> Option<()> {
        print!("Opening file...");
        self.open_file().map(|savedata: Vec<u8>| {
            println!("done");

            print!("Decrypting with XOR...");
            let xor_key: u8 = 11;
            let mut savefile_decrypted: Vec<u8> = Self::decrypt_xor(&savedata, xor_key);
            Self::remove_null_bytes(&mut savefile_decrypted);
            println!("done");

            print!("Decoding Base64...");
            let base64_decoded_savefile: Vec<u8> = URL_SAFE.decode(savefile_decrypted).unwrap();
            println!("done");

            print!("Decompressing with GZip...");
            let mut decompressed_data: Vec<u8> = Vec::new();
            GzDecoder::new(&base64_decoded_savefile[..])
                .read_to_end(&mut decompressed_data)
                .unwrap();
            println!("done");

            self.data = decompressed_data
        })
    }
}

pub enum DecoderVariant {
    NoFileSelected,
    FileSelected(FileSelected),
}

pub enum DecoderOutputResult {
    FileNotCreated,
    FileCreated,
}

pub struct Decoder {
    pub variant: DecoderVariant,
}

impl Default for Decoder {
    fn default() -> Self {
        Decoder {
            variant: DecoderVariant::NoFileSelected,
        }
    }
}

impl Decoder {
    pub fn new_no_file_selected() -> Self {
        Decoder {
            variant: DecoderVariant::NoFileSelected,
        }
    }

    pub fn new_file_selected(file_path: PathBuf) -> Self {
        let mut selected_file: FileSelected = FileSelected::new(file_path);
        selected_file.decode();
        Decoder {
            variant: DecoderVariant::FileSelected(selected_file),
        }
    }

    fn format_xml(src: &[u8]) -> Result<String, XmlReaderError> {
        print!("Formatting XML...");
        let mut dest: Vec<u8> = Vec::new();
        let reader: EventReader<&[u8]> = ParserConfig::new()
            .trim_whitespace(true)
            .ignore_comments(false)
            .create_reader(src);
        let mut writer: EventWriter<&mut Vec<u8>> = EmitterConfig::new()
            .perform_indent(true)
            .normalize_empty_elements(false)
            .autopad_comments(false)
            .create_writer(&mut dest);
        for event in reader {
            if let Some(event) = event?.as_writer_event() {
                writer.write(event).unwrap();
            }
        }
        println!("done");

        Ok(String::from_utf8(dest).unwrap())
    }

    pub fn output(&self) -> Result<DecoderOutputResult, Error> {
        match &self.variant {
            DecoderVariant::NoFileSelected => {
                Err(Error::new(ErrorKind::Other, "No file selected to save to"))
            }
            DecoderVariant::FileSelected(selected_file) => {
                if let Some(path) = FileDialog::new()
                    .set_directory("~")
                    .set_file_name(
                        format!("{:?}.xml", selected_file.file_name().unwrap())
                            .replace('\"', "")
                            .as_str(),
                    )
                    .save_file()
                {
                    let mut file = File::create(path).expect("Error creating file");
                    match Self::format_xml(selected_file.data.as_slice()) {
                        Ok(formatted_data) => {
                            file.write_all(formatted_data.as_bytes()).unwrap();
                            Ok(DecoderOutputResult::FileCreated)
                        }
                        Err(e) => {
                            println!("{}", e);
                            Ok(DecoderOutputResult::FileNotCreated)
                        }
                    }
                } else {
                    Ok(DecoderOutputResult::FileNotCreated)
                }
            }
        }
    }
}
