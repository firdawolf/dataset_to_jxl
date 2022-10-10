use clap::Parser;
use execute::Execute;
use haphazard::{AtomicPtr, HazardPointer};
use std::fs;
use std::fs::DirEntry;
use std::process::Command;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
// use voca_rs::Voca;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Source Folder directory without the / at the end
    #[arg(short, long)]
    input: String,
    /// Destination Folder directory without the / at the end
    #[arg(short, long)]
    output: String,
    /// Jpeg XL static exec folder directory with include the cjxl.exe at the end
    #[arg(short, long)]
    jpegxl_path: String,
    /// Number of concurrent exec to run (1 for 1 thread or 4 for 4 thread)
    #[arg(short, long, default_value_t = 4)]
    concurrent: i32,
}

fn main() {
    let args = Args::parse();
    let sourcefolder = args.input;
    let destinationfolder = args.output;
    let concurrent_amount = args.concurrent;
    let jpegxl_path = args.jpegxl_path;
    fs::create_dir_all(destinationfolder.clone()).expect("cannot create dir");
    let currentusage_arc = Arc::new(AtomicPtr::from(Box::new(0)));

    let destinationfolder_arc = Arc::new(destinationfolder.clone());
    let jpegxl_path_arc = Arc::new(jpegxl_path.clone());
    for mut currentdir in fs::read_dir(sourcefolder) {
        while let Some(thisentry) = currentdir.next() {
            match thisentry {
                Ok(direntry1) => {
                    let direntry_arc = Arc::new(direntry1);
                    let direntry = Arc::clone(&direntry_arc);
                    let currentusage = Arc::clone(&currentusage_arc);
                    let currentusage1 = Arc::clone(&currentusage_arc);
                    let mut h = HazardPointer::new();
                    if currentusage.safe_load(&mut h).expect("not null") < &concurrent_amount {
                        fs::create_dir_all(
                            destinationfolder.clone()
                                + "/"
                                + direntry.file_name().to_str().expect("msg"),
                        )
                        .expect("cannot create dir");
                        let destinationfolder_clone = Arc::clone(&destinationfolder_arc);
                        let jpegxl_path_clone = Arc::clone(&jpegxl_path_arc);

                        thread::spawn(move || {
                            run_task(direntry, destinationfolder_clone, jpegxl_path_clone);
                            let mut h1 = HazardPointer::new();
                            let my_x = currentusage1.safe_load(&mut h1).expect("not null");
                            currentusage1.store(Box::new(my_x - 1));
                        });
                        let my_x2 = currentusage.safe_load(&mut h).expect("not null");
                        currentusage.store(Box::new(my_x2 + 1));
                    } else {
                        let mut h2 = HazardPointer::new();
                        while currentusage.safe_load(&mut h2).expect("not null")
                            == &concurrent_amount
                        {
                            thread::sleep(Duration::from_secs(5))
                        }
                    }
                    // println!("{:?}", direntry.path());
                }
                Err(err) => println!("entry {} cannot be read", err),
            }
        }
    }

    println!("folder created");
}
fn run_task(direntry: Arc<DirEntry>, destinationfolder: Arc<String>, jpegxl_path: Arc<String>) {
    for mut insidedir in fs::read_dir(direntry.path()) {
        while let Some(insideentry) = insidedir.next() {
            match insideentry {
                Ok(file1) => {
                    let newfilename = file1
                        .file_name()
                        .into_string()
                        .expect("change to string file name failed");
                    let mut command = Command::new(&*jpegxl_path);
                    let destinationfile = destinationfolder.to_string()
                        + "/"
                        + direntry.file_name().to_str().expect("msg")
                        + "/"
                        + &newfilename.replace(".png", ".jxl");
                    command.args([
                        file1.path().to_str().expect("msg"),
                        destinationfile.as_str(),
                        "-q",
                        "95",
                        "--num_threads",
                        "1",
                        "-e",
                        "9",
                    ]);
                    if let Some(exit_code) = command.execute().unwrap() {
                        if exit_code == 0 {
                            println!("Ok.");
                        } else {
                            println!("Failed.");
                        }
                    } else {
                        println!("Interrupted!");
                    }
                }
                Err(err) => println!("entry {} cannot be read", err),
            }
        }
    }
}
