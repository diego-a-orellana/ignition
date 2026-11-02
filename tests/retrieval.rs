#![cfg(feature = "download")]

use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use flate2::{Compression, write::GzEncoder};
use ignition::{OpenCv, config::Config, retrieve::asset_retrieve};

const DEADLINE: Duration = Duration::from_secs(10);

/// The parent launches this ignored test as a separate downloader process.
#[test]
#[ignore]
fn retrieval_worker() {
    let address = std::env::var("IGNITION_TEST_ADDRESS").unwrap();
    let mut ready = TcpStream::connect(&address).unwrap();
    ready.set_read_timeout(Some(DEADLINE)).unwrap();
    ready.write_all(b"READY\n").unwrap();
    let mut start = [0];
    ready.read_exact(&mut start).unwrap();
    assert_eq!(start, [1]);
    let config = Config {
        bucket_url: format!("http://{address}"),
        build_dir: std::env::var("IGNITION_TEST_DIRECTORY").unwrap(),
        asset_dir: "assets".into(),
        cache_dir: "cache".into(),
        target: "test-target".into(),
    };
    asset_retrieve(&OpenCv::new(), &config).unwrap();
}

struct Worker(Child);

impl Worker {
    fn finish(&mut self) {
        let deadline = Instant::now() + DEADLINE;
        loop {
            if let Some(status) = self.0.try_wait().unwrap() {
                assert!(status.success(), "downloader exited with {status}");
                return;
            }
            assert!(Instant::now() < deadline, "downloader did not finish");
            thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        // Best-effort cleanup must also reap children when an assertion panics.
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

struct Fixture {
    directory: PathBuf,
    listener: TcpListener,
    archive: Vec<u8>,
    payload: Vec<u8>,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("ignition-{name}-{}-{stamp}", std::process::id()));
        fs::create_dir(&directory).unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let mut state = 17u32;
        let payload: Vec<_> = (0..65536)
            .map(|_| {
                state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                (state >> 24) as u8
            })
            .collect();
        let encoder = GzEncoder::new(Vec::new(), Compression::fast());
        let mut archive = tar::Builder::new(encoder);
        let mut header = tar::Header::new_gnu();
        header.set_mode(0o644);
        header.set_size(payload.len() as u64);
        header.set_cksum();
        archive
            .append_data(&mut header, "lib/payload", payload.as_slice())
            .unwrap();
        let archive = archive.into_inner().unwrap().finish().unwrap();
        Self {
            directory,
            listener,
            archive,
            payload,
        }
    }

    fn archive_path(&self) -> PathBuf {
        self.directory
            .join("cache/assets/opencv/test-target/opencv.tar.gz")
    }

    fn lock(&self) -> File {
        let path = self
            .directory
            .join("cache/assets/opencv/test-target/opencv.tar.gz.lock");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        File::options()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .unwrap()
    }

    /// Accept a bounded HTTP request or a worker's startup handshake.
    fn request(&self, expected: &str) -> TcpStream {
        let deadline = Instant::now() + DEADLINE;
        let mut stream = loop {
            match self.listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < deadline, "missing request: {expected}");
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("accepting {expected}: {error}"),
            }
        };
        stream.set_read_timeout(Some(DEADLINE)).unwrap();
        stream.set_write_timeout(Some(DEADLINE)).unwrap();
        let mut reader = BufReader::new(&mut stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert_eq!(line.split_whitespace().next(), Some(expected));
        if expected != "READY" {
            loop {
                line.clear();
                assert_ne!(reader.read_line(&mut line).unwrap(), 0);
                if line == "\r\n" {
                    break;
                }
            }
        }
        stream
    }

    fn start(&self) -> Worker {
        let child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "retrieval_worker", "--ignored", "--nocapture"])
            .env(
                "IGNITION_TEST_ADDRESS",
                self.listener.local_addr().unwrap().to_string(),
            )
            .env("IGNITION_TEST_DIRECTORY", &self.directory)
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
        let worker = Worker(child);
        self.request("READY").write_all(&[1]).unwrap();
        worker
    }

    /// Pause halfway through the body so another process overlaps the download.
    fn begin_download(&self) -> TcpStream {
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            self.archive.len()
        );
        self.request("HEAD").write_all(response.as_bytes()).unwrap();
        let mut body = self.request("GET");
        body.write_all(response.as_bytes()).unwrap();
        body.write_all(&self.archive[..self.archive.len() / 2])
            .unwrap();
        let partial = self
            .directory
            .join("cache/assets/opencv/test-target/opencv.tar.gz.part");
        let deadline = Instant::now() + DEADLINE;
        while !fs::metadata(&partial).is_ok_and(|metadata| metadata.len() > 0) {
            assert!(
                Instant::now() < deadline,
                "download did not create partial data"
            );
            thread::sleep(Duration::from_millis(10));
        }
        body
    }

    fn finish_download(&self, mut body: TcpStream) {
        body.write_all(&self.archive[self.archive.len() / 2..])
            .unwrap();
    }

    fn verify(&self) {
        assert_eq!(fs::read(self.archive_path()).unwrap(), self.archive);
        assert_eq!(
            fs::read(self.directory.join("assets/opencv/lib/payload")).unwrap(),
            self.payload
        );
        assert!(!self.archive_path().with_extension("gz.part").exists());
        assert_eq!(
            self.listener.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock,
            "a cached retrieval unexpectedly contacted the server"
        );
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if !thread::panicking() {
            fs::remove_dir_all(&self.directory).unwrap();
        }
    }
}

#[test]
fn concurrent_cold_cache_downloads_once() {
    let fixture = Fixture::new("cold");
    let mut first = fixture.start();
    let body = fixture.begin_download();
    assert!(matches!(
        fixture.lock().try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    let mut second = fixture.start();
    fixture.finish_download(body);
    first.finish();
    second.finish();
    fixture.verify();
}

#[test]
fn concurrent_warm_cache_extraction_uses_the_same_lock() {
    let fixture = Fixture::new("warm");
    let lock = fixture.lock();
    lock.lock().unwrap();
    fs::write(fixture.archive_path(), &fixture.archive).unwrap();
    let mut first = fixture.start();
    let mut second = fixture.start();
    assert!(first.0.try_wait().unwrap().is_none());
    assert!(second.0.try_wait().unwrap().is_none());
    drop(lock);
    first.finish();
    second.finish();
    fixture.verify();
}

#[test]
fn killed_downloader_releases_lock_and_partial_download_is_retried() {
    let fixture = Fixture::new("interrupted");
    let mut first = fixture.start();
    let body = fixture.begin_download();
    assert!(matches!(
        fixture.lock().try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    first.0.kill().unwrap();
    assert!(!first.0.wait().unwrap().success());
    drop(body);
    let lock = fixture.lock();
    lock.try_lock().unwrap();
    drop(lock);
    let mut successor = fixture.start();
    let body = fixture.begin_download();
    fixture.finish_download(body);
    successor.finish();
    fixture.verify();
}
