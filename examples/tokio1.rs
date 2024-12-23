use std::{
    thread::{self},
    time::Duration,
};

use tokio::{fs, runtime::Builder, time::sleep};

// #[tokio::main]
// async fn main() {
//     let a = 10;
//     let b = 20;
//     println!("{} + {} = {}", a, b, a + b);
// }
fn main() {
    let handle = thread::spawn(|| {
        // println!("Hello world!");
        let rt = Builder::new_current_thread().enable_all().build().unwrap();
        rt.spawn(async {
            println!("Future1");
            let content = fs::read("Cargo.toml").await.unwrap();
            println!("Future1 Content length: {}", content.len());
        });

        rt.spawn(async {
            println!("Future2");
            let ret = expensive_blocking_task("hello".to_string());
            println!("Future2 result: {}", ret);
        });

        rt.block_on(async {
            println!("Future3");
            sleep(Duration::from_millis(900)).await;
        })
    });
    handle.join().unwrap();
}

fn expensive_blocking_task(s: String) -> String {
    thread::sleep(Duration::from_millis(800));
    blake3::hash(s.as_bytes()).to_string()
}
