use flexi_logger::writers::LogWriter;

#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub level: log::Level,
}

pub struct VirtualLogger {
    records: tokio::sync::mpsc::Sender<Notification>,
}

impl VirtualLogger {
    pub fn new(records_sender: tokio::sync::mpsc::Sender<Notification>) -> Self {
        Self {
            records: records_sender,
        }
    }
}

impl LogWriter for VirtualLogger {
    fn write(
        &self,
        _: &mut flexi_logger::DeferredNow,
        record: &log::Record,
    ) -> std::io::Result<()> {
        let _ = tokio::task::block_in_place(|| {
            self.records.blocking_send(Notification {
                message: record.args().to_string(),
                level: record.level(),
            })
        });
        Ok(())
    }

    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }
}
