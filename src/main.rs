use std::sync::Arc;
use tokio::{
    net::{TcpListener, TcpStream},
    io::{
        BufReader,
        AsyncBufReadExt,
        AsyncWriteExt,
    },
    time::{
        Duration,
        Instant,
    },
    sync::{
        mpsc::{
            unbounded_channel,
            UnboundedSender, UnboundedReceiver,
        },
        Mutex,
    }
};
use serde::{Serialize, Deserialize};
use clap::Parser;

#[derive(Parser)]
struct CliOpts {
    #[arg(long, default_value = "127.0.0.1:4242")]
    addr: String,

    #[arg(long, default_value = "60")]
    sleep_timeout : u64,

    #[arg(long, default_value = "18446744073709551615")]
    max_count: u64,

    /// This parameter is for integration testing.
    #[arg(long, default_value = "false")]
    send_url_over_socket: bool,
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
        let counters : Arc<Mutex<(i128, i128)>> = Arc::new(Mutex::new((0, 0)));

        let timeout = Duration::from_secs(self.sleep_timeout);

        // To use sleep with select, you must pin the future.
        // https://docs.rs/tokio/latest/tokio/time/struct.Sleep.html#examples
        let sleep = tokio::time::sleep(timeout);
        tokio::pin!(sleep);

        for i in 0..self.max_count {
            tokio::select! {
                () = &mut sleep => {
                    //println!("Sending ppost request! with vals {odd_count}, {even_count}");
                    let client = reqwest::Client::new();
                    let (mut even_count, mut odd_count) = *counters.lock().await;
                    let body = format!("odd={odd_count} even={even_count}");
                    even_count = 0;
                    odd_count = 0;
                    sleep.as_mut().reset(Instant::now() + timeout);
                    if self.send_url_over_socket {
                    } else {
                        let req = client.post("https://paste.c-net.org/").body(body);
                        let text = req.send().await?.text().await?;
                        print!("{body} - {text}");
                    }
                    if i >= self.max_count - 1 {
                        return Ok(())
                    }
                }
                acceptor = listener.accept() => {
                    match acceptor {
                        Ok((socket, _addr)) => {
                            let counters = counters.clone();
                            let handle = tokio::spawn(async move {
                                let _ = handle_client(socket, counters).await;
                            });
                            handles.push(handle);
                        },
                        Err(e) => println!("couldn't get client: {:?}", e),
                    }
                }
            }
        }
        Ok(())
    }
}

// We do this because the problem prompt expliticly says don't use "fork".
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = CliOpts::parse();
    opts.run().await?;
    Ok(())

}
async fn handle_client(stream: TcpStream, counters: Arc<Mutex<(i128,i128)>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = BufReader::new(stream);
    let mut data = String::new();

    loop {
        stream.read_line(&mut data).await?;
        let lines : Vec<&str> = data.split('\n').collect();
        for line in &lines {
            if !line.is_empty() {
                let data : FluentData = if let Ok(data) = serde_json::from_str(line) {
                    data
                } else {
                    continue;
                };
                let (mut even_counter, mut odd_counter) = *counters.lock().await;
                if data.rand_value % 2 == 0 {
                    even_counter += 1;
                } else {
                    odd_counter += 1;
                }
                //let _ = stream.write_all(body.as_bytes());
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn single_client_test() {
        let addr = "127.0.0.1:4242";
        let opt_thread = tokio::spawn(async move {
            let opts = CliOpts {
                addr: addr.to_string(),
                sleep_timeout: 1,
                max_count: 10,
                send_url_over_socket: true,
            };
            let _ = opts.run().await;
        });
        // Must wait a few ms to ensure the server is listening.
        tokio::time::sleep(Duration::from_millis(100)).await;
        let mut stream = TcpStream::connect(addr).await.expect("Failed to connect to server");

        for i in 0..100 {
            let data = FluentData {
                date: 0.0,
                rand_value: i,
            };
            let json = serde_json::to_string(&data).expect("Failed to serialize to json");
            let json = format!("{json}\n");
            stream.write(json.as_bytes()).await.expect("Failed to write to stream");
            let _ = stream.flush().await;

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        println!("Ended send events!");

        let mut stream = BufReader::new(stream);
        let mut data = String::new();

        stream.read_line(&mut data).await.expect("Failed to read line");
        assert_eq!(data, "aoeu");

        let _ = opt_thread.await;
    }
}
