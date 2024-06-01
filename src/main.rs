use tokio::time::{Duration, Instant, sleep};
use sha1::{Sha1, Digest};
use tokio::task::JoinHandle;
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;
use hex;
use std::error::Error;
use tracing_subscriber::FmtSubscriber;
use tracing::{trace, info, Level, span, instrument};
use tokio::net::TcpStream;
use std::str;

async fn read_string_from_stream(stream: &mut TcpStream) -> Result<String, Box<dyn Error>> {
    stream.readable().await?;

    let mut buf = [0; 4096];
    let read = stream.read(&mut buf).await?;
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

#[instrument]
async fn emulator(id: String, target_hashrate: f64, ducoid: String) -> Result<(), Box<dyn Error>> {
//    let _span_ = span!(Level::INFO, id.clone()).entered();

    let mut stream = TcpStream::connect("212.132.102.74:8500").await?;
    let server_version = read_string_from_stream(&mut stream).await?;
    info!("server version: {}", &server_version[0..server_version.len() - 1]);

    loop {
        stream.writable().await?;
        stream.write_all("JOB,kadyklesha,AVR,lalala\n".as_bytes()).await?;
        let job = read_string_from_stream(&mut stream).await?;
        info!("got job: {}", &job[0..job.len() - 1]);

        let parts = job[0..job.len()-1].split(",").collect::<Vec<&str>>();
        let last_h = parts[0];
        let exp_h = parts[1];
        let diff = parts[2].parse::<u64>()?;

        let (res, hashrate) = mine(last_h.to_string(), exp_h.to_string(), diff, target_hashrate).await?;
        
        let ans = format!("{},{:.2},Official AVR Miner 4.0,{},{}", res, hashrate, id, ducoid);
        trace!("sending: {}", ans);

        stream.writable().await?;
        stream.write_all(ans.as_bytes()).await?;
        let status = read_string_from_stream(&mut stream).await?;
        info!("got answer: {}", &status[0..status.len() - 1]);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let mut handles: Vec<JoinHandle<()>> = vec![];

    for i in 0..10 {
        let handle = tokio::spawn(async move {
            let ducoids = vec![
                "DUCOID84e128aa5745db73".to_string(),
                "DUCOID40d745e4c4ca8b7c".to_string(),
                "DUCOID97ce67ac2a199f01".to_string(),
                "DUCOID29d6372681217a57".to_string(),
                "DUCOID44f1de2a1625c3f9".to_string(),
                "DUCOID1069e9ac5a23eed0".to_string(),
                "DUCOID1287436fbf3404e3".to_string(),
                "DUCOID2086b65f3c661ea0".to_string(),
                "DUCOID657ce340551a2a90".to_string(),
                "DUCOID3b093cf4e71ae617".to_string()
            ];

            let ids = vec![
                "my01".to_string(),
                "my02".to_string(),
                "my03".to_string(),
                "my04".to_string(),
                "my05".to_string(),
                "my06".to_string(),
                "my07".to_string(),
                "my08".to_string(),
                "my09".to_string(),
                "my10".to_string(),
            ];

            let hashrates = vec![
                298.4,
                295.2,
                300.1,
                290.4,
                302.1,
                310.5,
                304.7,
                300.0,
                297.9,
                308.7
            ];

            let _ = emulator(ids[i].clone(), hashrates[i], ducoids[i].clone()).await;
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await?;
    };
//    println!("{:#?}", algo("5c50c5631c92a9814220b1ed3709f0f05f4a34b1", hex!("27c1005102ba5fd9bb84347546999d1a7377cfda"), 100000, precalc))
    //println!("{:#?}", mine("205d7c95fdc2ce3e9bc682d82936ec4c4603e0c8".to_string(), "8ce9c115f23270fca847d60a4c13d597619f4a26".to_string(), 8, 305.0).await);

    Ok(())
}
