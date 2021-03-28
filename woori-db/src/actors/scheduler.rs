use actix::prelude::*;
use chrono::{Local, TimeZone, Utc};
use cron::Schedule;
use std::{
    fs::OpenOptions,
    io::{BufReader, BufWriter},
    path::PathBuf,
    process::Command,
    str::FromStr,
    time::Duration,
};

pub struct Scheduler;

impl Actor for Scheduler {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        log::debug!("Actor is alive");

        ctx.run_later(duration_until_next(), move |this, ctx| {
            this.schedule_task(ctx)
        });
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        log::debug!("Actor is stopped");
    }
}

impl Scheduler {
    fn schedule_task(&self, ctx: &mut Context<Self>) {
        use glob::glob;
        log::debug!("schedule_task event - {:?}", Local::now());
        let date_to_clear = Utc::now() - chrono::Duration::days(10);
        let files: Vec<PathBuf> = glob("*data/*.log")
            .unwrap()
            .map(std::result::Result::unwrap)
            .collect();

        files.iter().for_each(|f| {
            if let Some(file_name) = f.to_str() {
                let date = file_name.replace(".log", "").replace("data/", "");
                let file_date =
                    Utc.datetime_from_str(&format!("{} 00:00:00", date), "%Y_%m_%d %H:%M:%S");

                if file_date.is_ok() && file_date.unwrap() < date_to_clear {
                    use std::io;
                    let file_zip = file_name.replace(".log", ".zst");
                    let file = OpenOptions::new().read(true).open(file_name).unwrap();
                    let mut reader = BufReader::new(file);
                    let write = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .append(false)
                        .open(&file_zip)
                        .unwrap();
                    let writer = BufWriter::new(write);

                    let mut encoder = zstd::stream::Encoder::new(writer, 22).unwrap();
                    io::copy(&mut reader, &mut encoder).unwrap();
                    encoder.finish().unwrap();

                    Command::new("rm")
                        .arg("-rf")
                        .arg(&file_name)
                        .output()
                        .expect("Couldn't remove file");
                }
            }
        });

        ctx.run_later(duration_until_next(), move |this, ctx| {
            this.schedule_task(ctx)
        });
    }
}

pub fn duration_until_next() -> Duration {
    let cron_expression = "0 0 0 */1 * *"; // or "@daily"; every day at midnight
    let cron_schedule = Schedule::from_str(cron_expression).unwrap();
    let now = Local::now();
    let next = cron_schedule.upcoming(Local).next().unwrap();
    let duration_until = next.signed_duration_since(now);
    Duration::from_millis(duration_until.num_milliseconds() as u64)
}
