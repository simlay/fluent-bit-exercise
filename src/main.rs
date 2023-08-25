use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write, path::PathBuf};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    time::{Duration, Instant},
};

#[derive(Parser)]
struct CliOpts {
    #[arg(long, default_value = "127.0.0.1:4242")]
    addr: String,

    #[arg(long, default_value = "60")]
    sleep_timeout: u64,

    #[arg(long, default_value = "18446744073709551615")]
    max_count: u64,

    #[arg(long, default_value = "/dev/stdout")]
    out_file: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
struct FluentData {
    pub date: f64,
    pub rand_value: i128,
}

impl CliOpts {
    async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(self.addr).await?;
        let mut handles = Vec::new();
        let (sender, mut reciever): (UnboundedSender<i128>, UnboundedReceiver<i128>) =
            unbounded_channel();
        let (mut even_count, mut odd_count) = (0, 0);
        let mut file = File::create(self.out_file)?;

        let timeout = Duration::from_secs(self.sleep_timeout);

        // To use sleep with select, you must pin the future.
        // https://docs.rs/tokio/latest/tokio/time/struct.Sleep.html#examples
        let sleep = tokio::time::sleep(timeout);
        tokio::pin!(sleep);
        let mut request_count = 0;

        loop {
            tokio::select! {
                () = &mut sleep => {
                    let client = reqwest::Client::new();
                    let body = format!("odd={odd_count} even={even_count}");
                    even_count = 0;
                    odd_count = 0;
                    sleep.as_mut().reset(Instant::now() + timeout);
                    let req = client.post("https://paste.c-net.org/").body(body.clone());
                    if let Ok(resp) = req.send().await {
                        if let Ok(text) = resp.text().await {
                            file.write_all(text.to_string().as_bytes())?;
                        }
                    }
                    request_count += 1;
                    if request_count >= self.max_count - 1 {
                        return Ok(())
                    }
                }
                val = reciever.recv() => {
                    if let Some(val) = val {
                        if val % 2 == 0 {
                            even_count += 1;
                        } else {
                            odd_count += 1;
                        }
                    }
                }
                acceptor = listener.accept() => {
                    match acceptor {
                        Ok((socket, _addr)) => {
                            let sender = sender.clone();
                            let handle = tokio::spawn(async move {
                                let _ = handle_client(socket, sender).await;
                            });
                            handles.push(handle);
                        },
                        Err(e) => println!("couldn't get client: {:?}", e),
                    }
                }
            }
        }
    }
}

// We do this because the problem prompt expliticly says don't use "fork".
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = CliOpts::parse();
    opts.run().await?;
    Ok(())
}
async fn handle_client(
    stream: TcpStream,
    sender: UnboundedSender<i128>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = BufReader::new(stream);

    loop {
        let mut data = String::new();
        stream.read_line(&mut data).await?;
        if !data.is_empty() {
            let lines: Vec<&str> = data.split('\n').collect();

            for line in &lines {
                if !line.is_empty() {
                    let data: FluentData = if let Ok(data) = serde_json::from_str(line) {
                        data
                    } else {
                        continue;
                    };
                    let _ = sender.send(data.rand_value);
                }
            }
        }
    }
}
