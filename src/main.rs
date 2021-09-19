use lucet_runtime::{DlModule, Limits, MmapRegion, Region, Error};
use lucet_wasi::WasiCtxBuilder;
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // ensure the WASI symbols are exported from the final executable
    lucet_wasi::export_wasi_funcs();
    // load the compiled Lucet module
    let dl_module = DlModule::load("/root/hello-lucet/hello-rust.so").unwrap();
    // (*1) create a new memory region with default limits on heap and stack size
    let region = MmapRegion::create(
        1,
        &Limits::default().with_heap_memory_size(100 * 16 * 64 * 1024),
    ).unwrap();
    // instantiate the module in the memory region
    let mut instance = region.new_instance(dl_module).unwrap();
    // prepare the WASI context, inheriting stdio handles from the host executable
    let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().build();
    instance.insert_embed_ctx(wasi_ctx);
    // (*2) run the WASI main function
    let stream = TcpStream::connect("www.google.com:80").unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();

    let fd = stream.as_raw_fd();

    match instance.run_async("do_start", &[fd.into()], None).await {
      Err(Error::RuntimeTerminated(e)) => {eprintln!("{:?}", e)},
      Err(e) => {eprintln!("{}", e)},
      _ => {}
    }
}
