use base64::engine::{general_purpose::URL_SAFE, Engine};
use flate2::write::GzEncoder;
use flate2::Compression;
use rfd::FileDialog;
use std::fs::File;
use std::io::{Error, ErrorKind, Read, Write};
use std::path::PathBuf;
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

    fn encrypt_xor(data: &mut Vec<u8>, key: u8) -> &mut Vec<u8> {
        for byte in data.iter_mut() {
            *byte ^= key;
        }

        data
    }

    pub fn encode(&mut self) -> Option<()> {
        print!("Opening file...");
        self.open_file().map(|xmldata: Vec<u8>| {
            println!("done");

            print!("Compressing with GZip...");
            let mut encoder: GzEncoder<Vec<u8>> =
                GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&xmldata[..]).unwrap();
            let compressed_data: Vec<u8> = encoder.finish().unwrap();
            println!("done");

            print!("Encoding Base64...");
            let mut base64_encoded_savefile: Vec<u8> =
                URL_SAFE.encode(compressed_data).as_bytes().to_vec();
            println!("done");

            print!("Encrypting with XOR...");
            let xor_key: u8 = 11;
            let savefile_encrypted: &mut Vec<u8> =
                Self::encrypt_xor(&mut base64_encoded_savefile, xor_key);
            println!("done");

            self.data = savefile_encrypted.to_vec();
        })
    }
}

pub enum EncoderVariant {
    NoFileSelected,
    FileSelected(FileSelected),
}

pub enum EncoderOutputResult {
    FileNotCreated,
    FileCreated,
}

pub struct Encoder {
    pub variant: EncoderVariant,
}

impl Default for Encoder {
    fn default() -> Self {
        Encoder {
            variant: EncoderVariant::NoFileSelected,
        }
    }
}

impl Encoder {
    pub fn new_no_file_selected() -> Self {
        Encoder {
            variant: EncoderVariant::NoFileSelected,
        }
    }

    pub fn new_file_selected(file_path: PathBuf) -> Self {
        let mut selected_file: FileSelected = FileSelected::new(file_path);
        selected_file.encode();
        Encoder {
            variant: EncoderVariant::FileSelected(selected_file),
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

    pub fn output(&self) -> Result<EncoderOutputResult, Error> {
        match &self.variant {
            EncoderVariant::NoFileSelected => {
                Err(Error::new(ErrorKind::Other, "No file selected to save to"))
            }
            EncoderVariant::FileSelected(selected_file) => {
                if let Some(path) = FileDialog::new()
                    .set_directory("~")
                    .set_file_name(
                        format!("{:?}.dat", selected_file.file_name().unwrap())
                            .replace('\"', "")
                            .as_str(),
                    )
                    .save_file()
                {
                    let mut file = File::create(path).expect("Error creating file");
                    match Self::format_xml(selected_file.data.as_slice()) {
                        Ok(formatted_data) => {
                            file.write_all(formatted_data.as_bytes()).unwrap();
                            Ok(EncoderOutputResult::FileCreated)
                        }
                        Err(e) => {
                            println!("{}", e);
                            Ok(EncoderOutputResult::FileNotCreated)
                        }
                    }
                } else {
                    Ok(EncoderOutputResult::FileNotCreated)
                }
            }
        }
    }
}
