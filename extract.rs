use std::cmp::Ordering;
use std::fs;
use std::io::Write;
use std::path;
use std::io::{Read, Seek, SeekFrom};
use std::{fmt, io, num, str};

#[derive(Debug)]
pub enum DictError {
    Io(io::Error),
    Utf8(str::Utf8Error),
    Parse(num::ParseIntError),
    My(String),
    NotFound,
}

impl From<io::Error> for DictError {
    fn from(err: io::Error) -> Self {
        DictError::Io(err)
    }
}
impl From<str::Utf8Error> for DictError {
    fn from(err: str::Utf8Error) -> Self {
        DictError::Utf8(err)
    }
}
impl From<num::ParseIntError> for DictError {
    fn from(err: num::ParseIntError) -> Self {
        DictError::Parse(err)
    }
}
impl fmt::Display for DictError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            DictError::Io(err) => write!(f, "{}", err),
            DictError::Utf8(err) => write!(f, "{}", err),
            DictError::Parse(err) => write!(f, "{}", err),
            DictError::My(msg) => write!(f, "{}", msg),
            DictError::NotFound => write!(f, "not found"),
        }
    }
}


// file format:
// utf8-coded string with '\0' ending.
// offset in dict file u32 or u64(not used for now)
// length in dict file u32
const OFF_LEN_BYTES: usize = 8;
#[derive(Debug)]
pub struct Idx {
    content: Vec<u8>,//file content
    index: Vec<u32>,//end of each word
    //off_len_bytes: u32,
}

enum ParseState {
    Word,
    OffsetLength(u8),
}
struct Parser {
    off_len_bytes_m1: u8,
    state: ParseState,

    off_word: u32,
    result: Vec<u32>,
}
impl Parser {
    fn parse(&mut self, x: u8) {
        match self.state {
            ParseState::Word => {
                if x == 0 {
                    self.result.push(self.off_word);
                    self.state = ParseState::OffsetLength(0);
                }
            }
            ParseState::OffsetLength(n) => {
                self.state = if n < self.off_len_bytes_m1 {
                    ParseState::OffsetLength(n + 1)
                } else {
                    ParseState::Word
                };
            }
        }
        self.off_word += 1;
    }
}
impl Idx {
    pub fn open(file: &path::Path) -> Result<Idx, DictError> {
        let off_len_bytes = 8u8;
        let mut file_con: Vec<u8>;
        {
            let mut idx_file = fs::File::open(file)?;
            let filesize = idx_file.metadata()?;
            file_con = Vec::with_capacity(filesize.len() as usize + 1);//read to end may realloc...
            idx_file.read_to_end(&mut file_con)?;
        }
        let mut con = Parser {
            off_len_bytes_m1: off_len_bytes - 1,
            state: ParseState::Word,
            off_word: 0,
            result: Vec::with_capacity(10000),
        };
        file_con.iter().for_each(|x| con.parse(*x));

        //if count != con.result.len() {
        //    return Err(DictError::My(format!("not equal! {} != {}", count, con.result.len())));
        //}
        println!("idxfilesize={}", file_con.len());
        Ok(Idx { content:file_con, index: con.result })
    }
    pub fn len(&self) -> usize {
        self.index.len()
    }
    pub fn get_word(&self, i: usize) -> Result<&[u8], DictError> {
        //check range first
        if i >= self.index.len() {
            return Err(DictError::NotFound);
        }

        let start = if i == 0 { 0usize } else { self.index[i - 1] as usize + OFF_LEN_BYTES + 1 };
        let end = self.index[i] as usize;
        Ok(&self.content[start..end])
    }

    pub fn get_offset_length(&self, i: usize) -> Result<(u32, u32), DictError> {
        //check range first
        if i >= self.index.len() {
            return Err(DictError::NotFound);
        }

        let mut start = self.index[i] as usize + 1;
        let mut buff = [0u8; 4];
        buff.copy_from_slice(&self.content[start..start + 4]);
        let offset = u32::from_be_bytes(buff);
        start += 4;
        buff.copy_from_slice(&self.content[start..start + 4]);
        let length = u32::from_be_bytes(buff);
        Ok((offset, length))
    }
    // the result Err(usize) is used for neighborhood hint.
    pub fn get(&self, word: &[u8]) -> Result<usize, usize> {
        if Idx::dict_cmp(self.get_word(0).unwrap(), word, true) == Ordering::Greater {
            return Err(0);
        }
        if Idx::dict_cmp(self.get_word(self.index.len() - 1).unwrap(), word, true) == Ordering::Less {
            return Err(self.index.len());
        }

        let mut size = self.index.len();
        let mut base = 0usize;
        while size > 1 {
            let half = size / 2;
            let mid = base + half;
            // mid is always in [0, size), that means mid is >= 0 and < size.
            // mid >= 0: by definition
            // mid < size: mid = size / 2 + size / 4 + size / 8 ...
            let cmp = Idx::dict_cmp(self.get_word(mid).unwrap(), word, true);
            base = if cmp == Ordering::Greater { base } else { mid };
            size -= half;
        }
        // base is always in [0, size) because base <= mid.
        let cmp = Idx::dict_cmp(self.get_word(base).unwrap(), word, true);
        if cmp == Ordering::Equal { Ok(base) } else { Err(base + (cmp == Ordering::Less) as usize) }
    }
    pub fn dict_cmp(w1: &[u8], w2: &[u8], ignore_case: bool) -> Ordering {
        let w1len = w1.len();
        let w2len = w2.len();

        if w1len == 0 || w2len == 0 {
            return if w1len > 0 {
                Ordering::Greater
            } else if w2len > 0 {
                Ordering::Less
            } else {
                Ordering::Equal
            };
        }
        #[inline]
        fn i32_to_order(x: i32) -> Ordering {
            if x > 0 {
                Ordering::Greater
            } else if x < 0 {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        }
        let mut case_eq: i32 = 0;
        let mut ci2 = w2.iter();
        for c1 in w1.iter() {
            let c2: u8;
            match ci2.next() {
                None => return Ordering::Greater,
                Some(c) => c2 = *c,
            }
            let l2 = c2.to_ascii_lowercase();
            let l1 = c1.to_ascii_lowercase();
            if l1 > l2 {
                return Ordering::Greater;
            } else if l1 < l2 {
                return Ordering::Less;
            }
            if case_eq == 0 {
                case_eq = *c1 as i32 - c2 as i32;
                // do NOT return early.
            }
        }
        if w1len > w2len {
            Ordering::Greater
        } else if w1len < w2len {
            Ordering::Less
        } else if ignore_case {
            Ordering::Equal
        } else {
            i32_to_order(case_eq)
        }
    }
}
fn main() {
    let seperator1 = [1u8, 1, 3, b'\n'];
    let seperator2 = [5u8, 5, 3, b'\n'];
    let idxfile = Idx::open(&path::PathBuf::from("new.idx")).unwrap();

/*
    //idx dump
    let mut i = 0usize;
    println!("[");
    while i < idxfile.len() {
        let (offset,length) = idxfile.get_offset_length(i).unwrap();
        println!("{}\"word\":\"{}\",\"offset\":{},\"length\":{}{},", '{',
                 str::from_utf8(idxfile.get_word(i).unwrap()).unwrap(),
                 offset,
                 length,
                 '}');
        i += 1;
    }
    println!("]");
*/
    //dict extraction
    let mut dictfile = fs::File::open("new.dict").unwrap();
    let mut writer = fs::File::create("new.ext").unwrap();
    fn getdict(df: &mut fs::File, offset: u32, length: u32) -> Result<Vec<u8>, DictError> {
        df.seek(SeekFrom::Start(offset as u64))?;
        let mut result = vec![0u8; length as usize];
        df.read_exact(&mut *result)?;
        Ok(result)
    }
    let mut i = 0usize;
    while i < idxfile.len() {
        let (ioff, ilen) = idxfile.get_offset_length(i).unwrap();
        let key = idxfile.get_word(i).unwrap();
        let dict = getdict(&mut dictfile, ioff, ilen).unwrap();
        writer.write(&seperator1).unwrap();
        writer.write(key).unwrap();
        writer.write(&seperator2).unwrap();
        writer.write(&dict).unwrap();
        i += 1;
    }
    println!("wordcount={}", idxfile.len());

}
