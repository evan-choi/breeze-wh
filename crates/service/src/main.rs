mod service;
mod session;
mod supervisor;

fn main() -> anyhow::Result<()> {
    service::run()
}
