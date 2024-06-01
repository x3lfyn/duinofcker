use tokio::time::{Duration, Instant, sleep};
use sha1::{Sha1, Digest};
use tokio::io::AsyncWriteExt;
use hex;
use std::error::Error;
use tracing_subscriber::FmtSubscriber;
use tracing::{trace, info, Level};
use tokio::net::TcpStream;
use std::str;

async fn read_string_from_stream(stream: &TcpStream) -> Result<String, Box<dyn Error>> {
    stream.readable().await?;

    let mut buf = [0; 4096];
    let read = stream.try_read(&mut buf)?;
    let str = str::from_utf8(&buf[..read])?;

    Ok(str.to_string())
}

fn pow(last_h: String, exp_h: [u8; 20], diff: u64) -> u64 {
    let base = Sha1::new_with_prefix(last_h.as_bytes());
    for nonce in 0..(100 * diff + 1) {
        let mut temp = base.clone();
        temp.update(nonce.to_string().as_bytes());

        let digest = temp.finalize();
        if digest == exp_h.into() {
            return nonce
        }
    }

    return 0u64
}

async fn mine(last_h: String, exp_h: String, diff: u64, target_hashrate: f64) -> Result<(u64, f64), Box<dyn Error>> {
    let max_hashes = 100 * diff + 1;
    let decoded_exp_h = hex::decode(exp_h)?;

    let start_time = Instant::now();
    let res = pow(last_h, decoded_exp_h.try_into().unwrap(), diff);
    let elapsed_time = start_time.elapsed();

    trace!("elapsed time: {:?}", elapsed_time);

    let need_to_wait = if res == 0 {
        trace!("actual hashrate: {:.2} H/s", max_hashes as f64 / elapsed_time.as_secs_f64());
        trace!("result is zero, waiting and returning zero hashrate(imitated hashrate is zero)");
        Duration::from_millis((max_hashes as f64 / target_hashrate * 1000.0) as u64) - elapsed_time
    } else {
        trace!("actual hashrate: {:.2} H/s", res as f64 / elapsed_time.as_secs_f64());
        Duration::from_millis((res as f64 / target_hashrate * 1000.0) as u64) - elapsed_time
    };

    trace!("waiting {:?}", need_to_wait);
    sleep(need_to_wait).await;

    let imitated_hashrate = res as f64 / start_time.elapsed().as_secs_f64();

    trace!("imitated hashrate: {} H/s; delta {}", imitated_hashrate, imitated_hashrate - target_hashrate);

    return Ok((res, imitated_hashrate));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let mut stream = TcpStream::connect("212.132.102.74:8500").await?;
    let server_version = read_string_from_stream(&stream).await?;
    info!("server version: {}", &server_version[0..server_version.len() - 1]);

    stream.writable().await?;
    stream.write_all("JOB,nyaaa,AVR,lalala\r\n".as_bytes()).await?;
    let job = read_string_from_stream(&stream).await?;
    info!("got job: {}", &job[0..server_version.len() - 1]);
    
//    println!("{:#?}", algo("5c50c5631c92a9814220b1ed3709f0f05f4a34b1", hex!("27c1005102ba5fd9bb84347546999d1a7377cfda"), 100000, precalc))
    println!("{:#?}", mine("205d7c95fdc2ce3e9bc682d82936ec4c4603e0c8".to_string(), "8ce9c115f23270fca847d60a4c13d597619f4a26".to_string(), 8, 305.0).await);

    Ok(())
}
