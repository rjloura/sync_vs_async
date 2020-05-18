
// h/t to these folks for letting us hammer their server.
static URL: &str = "http://speedtest.tele2.net/1MB.zip";

use failure::Error;
use std::fs::File as StdFile;
use std::env;

use tokio::runtime::Runtime;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use futures::future::join_all;
use tokio::task::JoinHandle;

fn file_create(name: &str) -> StdFile {
    match StdFile::create(&name) {
        Err(e) => panic!("Error creating file {}", e),
        Ok(file) => file,
    }
}

fn sequential_download(num: u32) -> Result<(), Error> {
    let client = reqwest::blocking::Client::new();
    for i in 0..num {
        let filename = format!("file_sync{}", i);
        let mut response = client.get(URL).send()?;
        let mut file = file_create(&filename);

        std::io::copy(&mut response, &mut file)?;
    }
    Ok(())
}

async fn async_download(num: u32) -> Result<(), Error>{
    let mut tasks: Vec<JoinHandle<Result<(), Error>>> = vec![];
    for i in 0..num {
        tasks.push(tokio::spawn(async move {
            let filename = format!("file_async{}", i);
            let client = reqwest::Client::new();
            match client.get(URL).send().await {
                Ok(resp) => {
                    match resp.bytes().await {
                        Ok (r_bytes) => {
                            let mut file = File::create(filename).await?;
                            file.write_all(&r_bytes).await?;
                        },
                        Err(_) => println!("error downloading file"),
                    }
                },
                Err(_) => println!("error downloading file"),
            }
            Ok(())
        }));
    }
    join_all(tasks).await;
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut num_files = 10;

    if args.len() == 1 {
        println!("number of files not specified, using {}", num_files);
    } else if args.len() != 2 {
        println!("Usage: <bin> <num files>");
        std::process::exit(1);
    } else {
        num_files = args[1].parse().expect("parse num files arg");
    }

    println!("Testing with {} files", num_files);

    let now = std::time::Instant::now();
    Runtime::new()
        .expect("tokio runtime")
        .block_on(async_download(num_files)).expect("async download");
    println!("async_download: {}ms", now.elapsed().as_millis());

    let now = std::time::Instant::now();
    sequential_download(num_files).unwrap();
    println!("sequential_download: {}ms", now.elapsed().as_millis());
}
