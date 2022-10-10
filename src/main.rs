use haphazard::{AtomicPtr, Domain, HazardPointer};
use std::fs::DirEntry;
use std::process::Command;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::{env, f64::consts::E, fs};
use voca_rs::Voca;

use execute::Execute;

fn main() {
    let args: Vec<String> = env::args().collect();
    let sourcefolder = args[1].parse::<String>().expect("cannot convert first env");
    let destinationfolder = args[2]
        .parse::<String>()
        .expect("cannot convert second env");
    let concurrent_amount = args[3].parse::<i32>().expect("cannot convert third env");
    let jpegxl_path = args[4].parse::<String>().expect("cannot convert fifth env");
    fs::create_dir_all(destinationfolder.clone()).expect("cannot create dir");
    let currentusageArc = Arc::new(AtomicPtr::from(Box::new(0)));

    let destinationfolder_arc = Arc::new(destinationfolder.clone());
    let jpegxl_path_arc = Arc::new(jpegxl_path.clone());
    for mut currentdir in fs::read_dir(sourcefolder) {
        while let Some(thisentry) = currentdir.next() {
            match thisentry {
                Ok(direntry1) => {
                    let direntry_arc = Arc::new(direntry1);
                    let direntry = Arc::clone(&direntry_arc);
                    let currentusage = Arc::clone(&currentusageArc);
                    let currentusage1 = Arc::clone(&currentusageArc);
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
