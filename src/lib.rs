#[macro_use]
extern crate error_chain;

mod other_error {
    error_chain!{}
}



pub mod read_line {
    error_chain!{

        links {
            Another(other_error::Error, other_error::ErrorKind) #[cfg(unix)];
        }

    }

    use other_error;
    use std::fs::File;
    use std::io::prelude::*;
    use std::iter::*;

    enum State {
        CR,
        Other,
    }

    #[derive(PartialEq)]
    enum FileType {
        Win,
        Unix,
        Unknown,
    }



    pub struct LineReader {
        file: File,
        state: State,
        ft: FileType,
    }

    impl LineReader {
        pub fn new(file: File) -> Result<LineReader> {
            Ok(LineReader {
                file,
                state: State::Other,
                ft: FileType::Unknown,
            })
        }

        fn read_to_vec(&mut self) -> Result<Option<Vec<u8>>> {
            let file = &self.file;
            let mut line = Vec::new();
            for item in file.bytes() {

                let item = item.chain_err(|| "Error iterating over byte")?;

                if self.ft == FileType::Unix && item == b'\r' {
                    bail!("Found CR symbol in Unix-type file");
                }

                match item {
                    b'\r' => {
                        match self.state {
                            State::CR => bail!("Found two consecutive CR symbols"),
                            _ => self.state = State::CR,
                        }
                    },
                    b'\n' => {
                        match self.state {
                            State::CR => {
                                self.state = State::Other;
                                self.ft = FileType::Win;
                                return Ok(Some(line))
                            },
                            State::Other => {
                                match self.ft {
                                    FileType::Unix => {
                                        return Ok(Some(line))
                                    },
                                    FileType::Win => bail!("Found LF instead of CRLF in Windows-type file"),
                                    FileType::Unknown => {
                                        self.ft = FileType::Unix;
                                        return Ok(Some(line))
                                    },
                                }
                            },
                        }
                    },
                    _ => {
                        line.push(item);
                    },
                }
            }
            if line.is_empty() {
                Ok(None)
            }
            else {
                Ok(Some(line))
            }
        }

        fn read_to_string(&mut self) -> Result<Option<String>> {
            let vec = self.read_to_vec()
                .chain_err(|| "Error reading line into vec")?;
            let vec = match vec {
                Some(v) => v,
                None => return Ok(None),
            };
            let string = String::from_utf8(vec)
                .chain_err(|| "Error converting u8 to string")?;
            Ok(Some(string))
        }
    }

    impl Iterator for LineReader {
        type Item = Result<String>;
        fn next(&mut self) -> Option<Result<String>> {
            //converting Result<Option<>> to Option<Result<>>
            match self.read_to_string() {
                Ok(option) => match option {
                    Some(line) => Some(Ok(line)),
                    None => None,
                }
                Err(error) => Some(Err(error)),
            }
        }
    }

}



//#[cfg(test)]
//mod tests {
//    #[test]
//    fn it_works() {
//        assert_eq!(2 + 2, 4);
//    }
//}
