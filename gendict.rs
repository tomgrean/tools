use std::fs;
use std::io::Write;
use std::io::{self, Read, Seek, SeekFrom};
use std::path;

const SEPERATORK: [u8; 4usize] = [1u8, 1, 3, b'\n'];
const SEPERATORD: [u8; 4usize] = [5u8, 5, 3, b'\n'];

enum ParseState {
    PatternK(usize),
    PatternD(usize),
    Key,
    Dict,
}
struct Parser {
    buf: Vec<u8>,
    state: ParseState,
    fidx: fs::File,
    fdict: fs::File,
    count: u32,
}
impl Parser {
    fn parse(&mut self, x: u8) {
        match self.state {
            ParseState::PatternK(n) => {
                if x == SEPERATORK[n] {
                    self.state = if n < SEPERATORK.len() - 1 {
                        ParseState::PatternK(n + 1)
                    } else {
                        if self.buf.len() > 0 {
                            //save dict
                            self.fdict.write(&self.buf).unwrap();
                            //write length of idx
                            let dlen = self.buf.len() as u32;
                            self.fidx.write(&dlen.to_be_bytes()).unwrap();
                            self.buf.clear();
                        }
                        ParseState::Key
                    }
                } else {
                    //reject
                    self.state = ParseState::Dict;
                    self.buf.extend(&SEPERATORK[0..n]);
                    self.buf.push(x);
                }
            }
            ParseState::PatternD(n) => {
                if x == SEPERATORD[n] {
                    self.state = if n < SEPERATORD.len() - 1 {
                        ParseState::PatternD(n + 1)
                    } else {
                        if self.buf.len() > 0 {
                            self.buf.push(b'\0');
                            self.fidx.write(&self.buf).unwrap();
                            self.buf.clear();
                            //write offset.
                            let doff = self.fdict.seek(SeekFrom::Current(0)).unwrap();
                            let darr = (doff as u32).to_be_bytes();
                            self.fidx.write(&darr).unwrap();
                            self.count += 1;
                        }
                        ParseState::Dict
                    }
                } else {
                    self.state = ParseState::Key;
                    self.buf.extend(&SEPERATORD[0..n]);
                    self.buf.push(x);
                }
            }
            ParseState::Key => {
                if x == SEPERATORD[0] {
                    self.state = ParseState::PatternD(1);
                } else {
                    self.buf.push(x);
                }
            }
            ParseState::Dict => {
                if x == SEPERATORK[0] {
                    self.state = ParseState::PatternK(1);
                } else {
                    self.buf.push(x);
                }
            }
        }
    }
}

pub fn open(file: &path::Path, outidx: &path::Path, outdict: &path::Path) -> io::Result<()> {
    let mut file_con: Vec<u8>;
    {
        let mut dict_file = fs::File::open(file)?;
        let filesize = dict_file.metadata()?;
        file_con = Vec::with_capacity(filesize.len() as usize + 1); //read to end may realloc...
        dict_file.read_to_end(&mut file_con)?;
    }
    let mut con = Parser {
        buf: Vec::with_capacity(2000),
        state: ParseState::PatternK(0),
        fidx: fs::File::create(outidx)?,
        fdict: fs::File::create(outdict)?,
        count: 0,
    };
    file_con.iter().for_each(|x| con.parse(*x));
    //write the tail.
    if con.buf.len() > 0 {
        //save dict
        con.fdict.write(&con.buf).unwrap();
        //write length of idx
        let dlen = con.buf.len() as u32;
        con.fidx.write(&dlen.to_be_bytes()).unwrap();
        //con.buf.clear();
    }

    println!("StarDict's dict ifo file");
    println!("wordcount={}", con.count);
    println!("idxfilesize={}", con.fidx.seek(SeekFrom::Current(0)).unwrap());
    //if count != con.result.len() {
    //    return Err(DictError::My(format!("not equal! {} != {}", count, con.result.len())));
    //}
    Ok(())
}
fn main() -> io::Result<()> {
    open(
        &path::PathBuf::from("new.ext"),
        &path::PathBuf::from("new.idx"),
        &path::PathBuf::from("new.dict"),
    )
}
