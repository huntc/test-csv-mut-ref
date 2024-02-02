use std::io::BufWriter;

use chrono::{DateTime, Utc};
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

#[tokio::main]
async fn main() {
    // The objective of this example is to illustrate how a CSV writer can be
    // established for an asynchronous stream of events that, in our case, are
    // delivered via a channel. The stream can be sourced from many other things
    // than a channel of course.
    //
    // One particular use case is where the stream is sourced from an undefined
    // number of records, such as though read from a database. As each database
    // record is read, we then serialise to a CSV form and return that stream
    // as the body of an HTTP response. For example, if Axum is used, then a
    // response body can be established per:
    //
    //    let body = Body::from_stream(event_stream);

    let (events, events_receiver) = mpsc::channel::<(DateTime<Utc>, String)>(1);

    let mut csv = csv::Writer::from_writer(BufWriter::new(Vec::with_capacity(256)));
    let _ = csv.write_record(&["timestamp", "event"]);

    let mut event_stream = ReceiverStream::new(events_receiver).map(move |(timestamp, event)| {
        let _ = csv.write_field(timestamp.to_rfc3339());
        let _ = csv.write_field(event);
        let _ = csv.flush();
        let underlying_bytes = csv.get_mut().get_mut();
        let bytes = underlying_bytes.clone();
        underlying_bytes.clear();
        bytes
    });

    let _ = events.send((Utc::now(), String::from("some-event"))).await;
    drop(events);

    while let Some(bytes) = event_stream.next().await {
        let record = String::from_utf8(bytes).unwrap();
        println!("{record}");
    }
}
