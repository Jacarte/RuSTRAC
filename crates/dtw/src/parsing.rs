//! Parsing module
//! Main functionality: to parse two sequences and create two arrays with fixed ids, numbers that
//! is what we use
//!
//!

use crate::dtw::*;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

pub trait TraceEncoder<'a> {
    /// Creates a trace bin file
    fn create_bin(&mut self, tokens: Vec<String>, to: PathBuf) -> Vec<TokenID>;

    /// Filter the sequence for interesting events. It returns a trasnformed String with
    /// uninteresting characters already removed.
    /// This is helpful to compare machine code, where we are only interested in the instructions
    /// and not the addresses
    fn filter(&self, token: String) -> String;

    /// Maps the token to a unique id
    fn token_to_id(&mut self, token: &str) -> TokenID;

    /// Maps the id to the token.
    /// This is helpful to generate the alignment
    fn id_to_token(&self, id: TokenID) -> String;

    /// Loads a trace bin from file
    fn deserialize(&self, from: PathBuf) -> Box<dyn Accesor>;
}

#[derive(Default)]
pub struct ToMemoryParser {
    // Global maps
    token_to_id: HashMap<String, TokenID>,
    id_to_token: HashMap<TokenID, String>,
}

impl<'a> TraceEncoder<'a> for ToMemoryParser {
    fn create_bin(&mut self, tokens: Vec<String>, to: PathBuf) -> Vec<TokenID> {
        // The tokens are already extracted...the extractor is a regular split
        // The default implementation is to get one token per line
        let mut r = vec![];

        for t in tokens {
            r.push(self.token_to_id(&self.filter(t)))
        }

        // Write the trace file
        // First 4 bytes the header 'dtw\0'
        // Second 4 bytes the version of this tool 0x00000001
        // Third 4 bytes the size f the vector
        // Then 4 bytes per token (the TypeID should fit into 4 bytes)
        //
        let f = std::fs::File::create(to).expect("File coudl not be created");
        let mut bw = std::io::BufWriter::new(f);

        // Write the header
        bw.write_all(b"dtw\0").expect("Could not write the header");
        bw.write_all(&[0x00, 0x00, 0x00, 0x01])
            .expect("Could not write the version");
        let _ = bw.write_all(&(r.len() as u32).to_le_bytes());
        // Write the bytes
        for i in &r {
            bw.write_all(&i.to_le_bytes())
                .expect("Could not write the token");
        }

        r
    }

    fn filter(&self, token: String) -> String {
        // All allowed for now
        // TODO implement a regular expression filter here
        token.clone()
    }

    fn token_to_id(&mut self, token: &str) -> TokenID {
        let id = self.token_to_id.len();
        // Is the size of the dict when inserting if it does not exist
        let id = *self
            .token_to_id
            .entry(token.to_string())
            .or_insert(id as TokenID);

        // Insert in the id to token with the inverse value
        self.id_to_token.insert(id as TokenID, token.to_string());

        // Return the id
        id as TokenID
    }

    fn id_to_token(&self, id: TokenID) -> String {
        return self.id_to_token.get(&id).unwrap().clone();
    }

    fn deserialize(&self, from: PathBuf) -> Box<dyn Accesor> {
        let f = std::fs::File::open(from).expect("File coudl not be opened");

        let mut br = std::io::BufReader::new(f);
        // Read the header
        // First 4 bytes the header 'dtw\0'
        // Read as bytes
        let header: [u8; 4] = {
            let mut r = [0; 4];
            br.read_exact(&mut r).unwrap();
            r
        };
        assert_eq!(&header, b"dtw\0");
        let version = br.read_u32::<BigEndian>().unwrap();
        assert_eq!(version, 0x00000001);

        let count = br.read_u32::<LittleEndian>().unwrap();
        let mut r = vec![];

        // 8 bytes per ID...that is too much :|
        for _ in 0..count {
            r.push(br.read_u64::<LittleEndian>().unwrap() as TokenID);
        }

        Box::new(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing_unparsing() {
        let mut parser = ToMemoryParser {
            id_to_token: HashMap::new(),
            token_to_id: HashMap::new(),
        };

        let tokens = vec![
            String::from("add 2,2"),
            String::from("sub 2 2"),
            String::from("mul 2,2"),
            String::from("sub 2 2"),
        ];

        let tokens = parser.create_bin(tokens, PathBuf::from("test.bin"));

        println!("{:?}", tokens);

        let accessor = parser.deserialize(PathBuf::from("test.bin"));

        for i in 0..tokens.len() {
            assert_eq!(tokens[i], accessor.get(i) as TokenID);
        }
    }
}
