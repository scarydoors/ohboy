use std::{env, fs::File, io::Read, path, process::ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let file = match args.get(1) {
        Some(s) => {
            let filepath = path::Path::new(s);
            File::open(filepath)                    
        },
        None => {
            eprintln!("missing path argument");
            return ExitCode::from(2);
        },
    };

    let mut file = match file {
        Ok(file) => file,
        Err(err) => {
            eprintln!("unable to open file: {}", err);
            return ExitCode::FAILURE
        },
    };

    let mut next_2_bytes: [u8; 2] = [0; 2];
    file.read_exact(&mut next_2_bytes).unwrap();

    println!("{:#X}", next_2_bytes.get(0).unwrap());
    println!("{:#X}", next_2_bytes.get(1).unwrap());

    ExitCode::SUCCESS
}
