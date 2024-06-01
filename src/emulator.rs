use hex;
use crate::util::read_string_from_stream;
use crate::algorithm::pow;
use tokio::net::TcpStream;
use std::error::Error;
use tokio::io::AsyncWriteExt;
use tokio::time::{sleep, Duration, Instant};
use tracing::{info, trace, span, Level, Span};


pub async fn mine(last_h: String, exp_h: String, diff: u64, target_hashrate: f64, span: Span) -> Result<(u64, f64), Box<dyn Error>> {
    let span = span.enter();

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

pub async fn emulator(id: &String, hashrate: f64, username: &String, mining_key: &String) -> Result<(), Box<dyn Error>> {
    let span = span!(Level::INFO, "emulator", ?id);

    let ducoid = format!("DUCOID{:x?}", rand::random::<u64>());

    let mut stream = TcpStream::connect("212.132.102.74:8500").await?;
    let server_version = read_string_from_stream(&mut stream).await?;
    span.in_scope(|| {
        info!("server version: {}", &server_version[0..server_version.len() - 1]);
    });

    loop {
        stream.writable().await?;
        stream.write_all(format!("JOB,{},AVR,{}\n", username, mining_key).as_bytes()).await?;
        let job = read_string_from_stream(&mut stream).await?;

        span.in_scope(|| {
            info!("got job: {}", &job[0..job.len() - 1]);
        });

        let parts = job[0..job.len()-1].split(",").collect::<Vec<&str>>();
        let last_h = parts[0];
        let exp_h = parts[1];
        let diff = parts[2].parse::<u64>()?;

        let (res, hashrate) = mine(last_h.to_string(), exp_h.to_string(), diff, hashrate, span.clone()).await?;
        
        let ans = format!("{},{:.2},Official AVR Miner 4.0,{},{}", res, hashrate, id, ducoid);
        span.in_scope(|| {
            trace!("sending: {}", ans);
        });

        stream.writable().await?;
        stream.write_all(ans.as_bytes()).await?;
        let status = read_string_from_stream(&mut stream).await?;
        span.in_scope(|| {
            info!("got answer: {}", &status[0..status.len() - 1]);
        });
    }
}
